// 调用 tauri-rust后端函数
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  VaultInfo,
  NoteMeta,
  LinkRef,
  Backlink,
  GraphData,
  SearchResult,
  SearchMode,
  FileChangeEvent,
  WatcherErrorEvent,
  CreateNoteResult
} from './types'

export async function openVault(path: string): Promise<VaultInfo> {
  return invoke('open_vault', { path })
}

export async function addVaultByPath(path: string): Promise<VaultInfo> {
  return invoke('add_vault_by_path', { path })
}

export async function listVaults(): Promise<VaultInfo[]> {
  return invoke('list_vaults')
}

export async function switchVault(vaultId: string): Promise<VaultInfo> {
  return invoke('switch_vault', { vaultId })
}

export async function closeVault(): Promise<void> {
  return invoke('close_vault')
}

export async function removeVault(vaultId: string): Promise<VaultInfo> {
  return invoke('remove_vault', { vaultId })
}

export async function getCurrentVault(): Promise<VaultInfo | null> {
  return invoke('get_current_vault')
}

export async function getWatcherStatus(): Promise<boolean> {
  return invoke('get_watcher_status')
}

export async function listAllNotes(): Promise<NoteMeta[]> {
  return invoke('list_all_notes')
}

export async function getNoteMeta(path: string): Promise<NoteMeta | null> {
  return invoke('get_note_meta', { path })
}

export async function getBacklinks(path: string): Promise<Backlink[]> {
  return invoke('get_backlinks', { path })
}

export async function getOutlinks(path: string): Promise<LinkRef[]> {
  return invoke('get_outlinks', { path })
}

export async function getUnresolvedLinks(path: string): Promise<LinkRef[]> {
  return invoke('get_unresolved_links', { path })
}

export async function getAllTags(): Promise<string[]> {
  return invoke('get_all_tags')
}

export async function resolveWikilink(target: string): Promise<string | null> {
  return invoke('resolve_wikilink', { target })
}

export async function renameNote(oldPath: string, newPath: string): Promise<void> {
  return invoke('rename_note', { oldPath, newPath })
}

export async function deleteNote(path: string): Promise<void> {
  return invoke('delete_note', { path })
}

export async function deleteFolder(path: string): Promise<void> {
  return invoke('delete_folder', { path })
}

export async function search(
  query: string,
  mode: SearchMode = 'fulltext',
  limit?: number
): Promise<SearchResult[]> {
  return invoke('search', { query, mode, limit })
}

export async function reindexAll(): Promise<void> {
  return invoke('reindex_all')
}

export async function getGraph(): Promise<GraphData> {
  return invoke('get_graph')
}

export async function onFileChanged(callback: (event: FileChangeEvent) => void): Promise<() => void> {
  const unlisten = await listen('roc://file-changed', (event: { payload: FileChangeEvent }) => {
    callback(event.payload)
  })
  return () => unlisten()
}

export async function onIndexUpdated(callback: (path: string) => void): Promise<() => void> {
  const unlisten = await listen('roc://index-updated', (event: { payload: { note_path: string } }) => {
    callback(event.payload.note_path)
  })
  return () => unlisten()
}

export async function onWatcherError(callback: (error: WatcherErrorEvent) => void): Promise<() => void> {
  const unlisten = await listen('roc://watcher-error', (event: { payload: WatcherErrorEvent }) => {
    callback(event.payload)
  })
  return () => unlisten()
}

export async function onVaultOpened(callback: (vault: VaultInfo) => void): Promise<() => void> {
  const unlisten = await listen('roc://vault-opened', (event: { payload: VaultInfo }) => {
    callback(event.payload)
  })
  return () => unlisten()
}

export async function onVaultClosed(callback: () => void): Promise<() => void> {
  const unlisten = await listen('roc://vault-closed', () => {
    callback()
  })
  return () => unlisten()
}

export async function createNote(): Promise<CreateNoteResult> {
  return invoke('create_note')
}

export async function createFolder(): Promise<{ path: string; name: string }> {
  return invoke('create_folder')
}

export async function deleteFolder(path: string): Promise<void> {
  return invoke('delete_folder', { path })
}