<script setup lang="ts">
import { computed } from 'vue'
import markdownit from 'markdown-it'
import markdownItKatex from '@vscode/markdown-it-katex'
import markdownItTaskLists from 'markdown-it-task-lists'
import hljs from 'highlight.js'
import { useNotesStore } from '../../stores/notes'

const notesStore = useNotesStore()

const md = markdownit({
  html: false,
  linkify: true,
  highlight: (str: string, lang: string | undefined) => {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value
      } catch (__) {}
    }
    return hljs.highlightAuto(str).value
  }
})
  .use(markdownItKatex)
  .use(markdownItTaskLists, { enabled: true })

md.inline.ruler.before('link', 'wikilink', (state: any, silent: boolean) => {
  const start = state.pos
  const max = state.posMax

  if (state.src.charCodeAt(start) !== 0x5B || state.src.charCodeAt(start + 1) !== 0x5B) {
    return false
  }

  let end = start + 2
  let depth = 1
  let found = false

  while (end < max) {
    const ch = state.src.charCodeAt(end)
    if (ch === 0x5D && state.src.charCodeAt(end + 1) === 0x5D) {
      depth--
      if (depth === 0) {
        found = true
        end += 2
        break
      }
    } else if (ch === 0x5B && state.src.charCodeAt(end + 1) === 0x5B) {
      depth++
    }
    end++
  }

  if (!found) {
    return false
  }

  if (!silent) {
    const content = state.src.slice(start + 2, end - 2)
    const parts = content.split(/[#|]/)
    const target = parts[0]
    const alias = parts.length > 2 ? parts[2] : target

    const token = state.push('wikilink', 'wikilink', 0)
    token.meta = { target, alias }
  }

  state.pos = end
  return true
})

md.renderer.rules.wikilink = (tokens: any[], idx: number) => {
  const token = tokens[idx]
  const target = token.meta.target
  const alias = token.meta.alias
  return `<a class="wikilink" data-target="${target}">${alias}</a>`
}

md.inline.ruler.after('wikilink', 'hashtag', (state: any, silent: boolean) => {
  const start = state.pos
  const max = state.posMax

  if (state.src.charCodeAt(start) !== 0x23) {
    return false
  }

  let end = start + 1
  const re = /^[a-zA-Z][a-zA-Z0-9_-]*/
  const match = state.src.slice(start + 1).match(re)
  
  if (!match) {
    return false
  }

  end = start + 1 + match[0].length

  if (!silent) {
    const token = state.push('hashtag', 'hashtag', 0)
    token.meta = { tag: match[0] }
  }

  state.pos = end
  return true
})

md.renderer.rules.hashtag = (tokens: any[], idx: number) => {
  const token = tokens[idx]
  const tag = token.meta.tag
  return `<a class="hashtag" data-tag="${tag}">#${tag}</a>`
}

const renderedContent = computed(() => {
  return md.render(notesStore.currentContent)
})

function handleClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  const wikilink = target.closest('.wikilink')
  const hashtag = target.closest('.hashtag')

  if (wikilink) {
    const notePath = wikilink.getAttribute('data-target')
    if (notePath) {
      notesStore.openNote(`${notePath}.md`)
    }
  } else if (hashtag) {
    const tag = hashtag.getAttribute('data-tag')
    console.log('Tag clicked:', tag)
  }
}
</script>

<template>
  <div class="preview-pane" @click="handleClick">
    <div class="preview-content" v-html="renderedContent"></div>
  </div>
</template>