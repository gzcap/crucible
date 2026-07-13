import type { Ctx, MilkdownPlugin } from '@milkdown/kit/ctx'
import { prosePluginsCtx } from '@milkdown/kit/core'
import { Plugin, PluginKey } from '@milkdown/kit/prose/state'
import { Decoration, DecorationSet } from '@milkdown/kit/prose/view'
/**
 * Wikilink Milkdown 插件：将 [[笔记名]] 文本渲染为可点击的链接样式
 * 不改变文档内容（仍是纯文本），只通过 ProseMirror decoration 改变视觉呈现
 * 支持 [[目标|别名]]、[[目标#标题]] 语法
 */

/**
 * 注意：不使用模块级带 g 标志的正则常量，避免 lastIndex 在多次调用间共享导致匹配异常。
 * 每次需要匹配时创建局部正则。
 */
const WIKILINK_PATTERN = () => /\\?\[\\?\[([^\]\n#|]+)(#[^\]|]*)?(\|[^\]]*)?\\?\]\\?\]/g

function createWikilinkPlugin() {
  return new Plugin({
    key: new PluginKey('roc-wikilink'),
    state: {
      init(_, state) {
        return buildDecorations(state)
      },
      apply(tr, set) {
        set = set.map(tr.mapping, tr.doc)
        if (!tr.docChanged) return set
        return buildDecorationsFromDoc(tr.doc)
      },
    },
    props: {
      decorations(state) {
        return this.getState(state)
      },
    },
  })
}

function buildDecorations(state: any) {
  return buildDecorationsFromDoc(state.doc)
}

function buildDecorationsFromDoc(doc: any) {
  try {
    const decorations: Decoration[] = []
    const docSize = doc.content.size

    doc.descendants((node: any, pos: number) => {
      if (!node.isText) return

      const text = node.text || ''
      const regex = WIKILINK_PATTERN()
      let match: RegExpExecArray | null

      while ((match = regex.exec(text)) !== null) {
        const from = pos + match.index
        const to = from + match[0].length
        // 防御：确保位置有效，避免 ProseMirror 状态异常
        if (from < 0 || to <= from || to > docSize + 1) continue
        const target = match[1].replace(/^\\/, '')

        decorations.push(
          Decoration.inline(from, to, {
            class: 'roc-wikilink',
            'data-target': target,
            title: `打开 ${target}`,
          }),
        )
      }
    })

    return DecorationSet.create(doc, decorations)
  } catch (e) {
    console.error('[wikilink] buildDecorations error:', e)
    return DecorationSet.empty
  }
}

export const wikilink: MilkdownPlugin = (ctx: Ctx) => {
  ctx.update(prosePluginsCtx, (prev) => [...prev, createWikilinkPlugin()])

  return () => {
    ctx.update(prosePluginsCtx, (prev) =>
      prev.filter((p) => (p as any)?.key?.key !== 'roc-wikilink'),
    )
  }
}
