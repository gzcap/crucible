<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { FolderOpened } from '@element-plus/icons-vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useVaultStore } from '../../stores/vault'

const vaultStore = useVaultStore()
const selectedVault = ref<string | null>(null)

onMounted(async () => {
  await vaultStore.loadVaults()
})

async function selectFolder() {
  const result = await open({
    directory: true,
    multiple: false
  })

  if (result) {
    const path = Array.isArray(result) ? result[0] : result
    await vaultStore.open(path)
  }
}

async function openExistingVault(vaultId: string) {
  await vaultStore.switchTo(vaultId)
}
</script>

<template>
  <div class="vault-picker">
    <div class="vault-picker-content">
      <h1 class="vault-title">Roc</h1>
      <p class="vault-subtitle">Your personal knowledge base</p>

      <el-button type="primary" size="large" @click="selectFolder" class="primary-btn">
        <FolderOpened :size="18" />
        Open Vault
      </el-button>

      <div v-if="vaultStore.vaults.length > 0" class="vault-list">
        <h2>Recent Vaults</h2>
        <el-card v-for="vault in vaultStore.vaults" :key="vault.id" class="vault-card">
          <div
            class="vault-item"
            :class="{ selected: selectedVault === vault.id }"
            @click="selectedVault = vault.id"
          >
            <span class="vault-icon">📂</span>
            <div class="vault-info">
              <span class="vault-name">{{ vault.name }}</span>
              <span class="vault-path">{{ vault.path }}</span>
            </div>
            <el-button type="primary" size="small" @click.stop="openExistingVault(vault.id)">
              Open
            </el-button>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>
