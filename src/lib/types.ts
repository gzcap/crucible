export interface VaultInfo {
  id: string
  name: string
  path: string
  last_opened?: string
}

export interface NoteMeta {
  title: string
  path: string
  tags: string[]
  mtime: number
  size: number
}

export interface LinkRef {
  target: string
  alias?: string
  heading?: string
}

export interface Backlink {
  source: string
  source_title: string
  snippet: string
}

export interface GraphNode {
  id: string
  name: string
  tag_count: number
  link_count: number
}

export interface GraphLink {
  source: string
  target: string
}

export interface GraphData {
  nodes: GraphNode[]
  links: GraphLink[]
}

export interface SearchResult {
  score: number
  path: string
  title: string
  snippet: string
}

export type SearchMode = 'fulltext' | 'tag' | 'link'

export interface FileChangeEvent {
  kind: 'create' | 'delete' | 'rename' | 'modify'
  path: string
  new_path?: string
}

export interface WatcherErrorEvent {
  message: string
}

export interface EditorTab {
  path: string
  title: string
  dirty: boolean
  type?: 'note' | 'graph'
}

export type EditorMode = 'wysiwyg' | 'source'

export interface CreateNoteResult {
  path: string
  title: string
}