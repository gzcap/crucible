<script setup lang="ts">
import { computed } from 'vue'
import { FolderOpened, Plus } from '@element-plus/icons-vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useVaultStore } from '../../stores/vault'

const vaultStore = useVaultStore()

const currentVault = computed(() => vaultStore.currentVault)

async function selectNewVault() {
  const result = await open({
    directory: true,
    multiple: false
  })

  if (result) {
    const path = Array.isArray(result) ? result[0] : result
    await vaultStore.open(path)
  }
}

async function switchVault(vaultId: string) {
  await vaultStore.switchTo(vaultId)
}
</script>

<template>
  <el-dropdown @command="switchVault" trigger="click">
    <span class="vault-selector">
      <FolderOpened :size="16" />
      <span class="vault-name">{{ currentVault?.name || '请选择仓库' }}</span>
      <el-icon class="dropdown-arrow">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="m6 9 6 6 6-6"/>
        </svg>
      </el-icon>
    </span>
    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item
          v-for="vault in vaultStore.vaults"
          :key="vault.id"
          :command="vault.id"
          :disabled="vault.id === currentVault?.id"
        >
          <FolderOpened :size="14" />
          {{ vault.name }}
        </el-dropdown-item>
        <el-dropdown-divider />
        <el-dropdown-item command="add" @click.native="selectNewVault">
          <Plus :size="14" />
          打开文件夹
        </el-dropdown-item>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>
