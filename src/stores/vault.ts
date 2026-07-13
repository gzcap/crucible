import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { VaultInfo } from '../lib/types'
import { openVault, addVaultByPath, listVaults, switchVault, closeVault, removeVault, getCurrentVault } from '../lib/tauri'

export const useVaultStore = defineStore('vault', () => {
  const vaults = ref<VaultInfo[]>([])
  const currentVault = ref<VaultInfo | null>(null)
  const isLoading = ref(false)

  async function loadVaults() {
    vaults.value = await listVaults()
  }

  async function open(path: string) {
    isLoading.value = true
    try {
      currentVault.value = await openVault(path)
      await loadVaults()
    } finally {
      isLoading.value = false
    }
  }

  async function add(path: string) {
    const vault = await addVaultByPath(path)
    vaults.value.push(vault)
  }

  async function switchTo(vaultId: string) {
    isLoading.value = true
    try {
      currentVault.value = await switchVault(vaultId)
      await loadVaults()
    } finally {
      isLoading.value = false
    }
  }

  async function close() {
    await closeVault()
    currentVault.value = null
  }

  async function remove(vaultId: string) {
    await removeVault(vaultId)
    vaults.value = vaults.value.filter(v => v.id !== vaultId)
    if (currentVault.value?.id === vaultId) {
      currentVault.value = null
    }
  }

  async function init() {
    await loadVaults()
    currentVault.value = await getCurrentVault()
  }

  return {
    vaults,
    currentVault,
    isLoading,
    loadVaults,
    open,
    add,
    switchTo,
    close,
    remove,
    init
  }
})
