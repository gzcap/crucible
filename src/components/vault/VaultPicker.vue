<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { FolderOpened, Plus, Clock } from '@element-plus/icons-vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useVaultStore } from '../../stores/vault'
import { useNotesStore } from '../../stores/notes'

const router = useRouter()
const vaultStore = useVaultStore()
const notesStore = useNotesStore()
const selectedVault = ref<string | null>(null)

onMounted(async () => {
  await vaultStore.loadVaults()
})

const recentVaults = computed(() => {
  return vaultStore.vaults.slice(0, 5)
})

const otherVaults = computed(() => {
  return vaultStore.vaults.slice(5)
})

async function selectFolder() {
  const result = await open({
    directory: true,
    multiple: false
  })

  if (result) {
    const path = Array.isArray(result) ? result[0] : result
    await vaultStore.open(path)
    await notesStore.loadNotes()
    await router.push("/")
  }
}

async function openExistingVault(vaultId: string) {
  await vaultStore.switchTo(vaultId)
  await notesStore.loadNotes()
  await router.push("/")
}

function formatDate(dateStr: string | undefined): string {
  if (!dateStr) return ''
  try {
    const date = new Date(dateStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))
    if (days === 0) return '今天'
    if (days === 1) return '昨天'
    if (days < 7) return `${days}天前`
    if (days < 30) return `${Math.floor(days / 7)}周前`
    return `${Math.floor(days / 30)}月前`
  } catch {
    return ''
  }
}
</script>

<template>
  <div class="vault-picker">
    <div class="vault-picker-content">
      <h1 class="vault-title">Roc</h1>
      <p class="vault-subtitle">Your personal knowledge base</p>

      <el-button type="primary" size="large" @click="selectFolder" class="primary-btn">
        <Plus :size="18" />
        Open Vault
      </el-button>

      <div v-if="vaultStore.vaults.length > 0" class="vault-list">
        <div v-if="recentVaults.length > 0" class="vault-section">
          <div class="section-header">
            <Clock :size="14" />
            <h2>最近打开</h2>
          </div>
          <el-card v-for="vault in recentVaults" :key="vault.id" class="vault-card">
            <div
              class="vault-item"
              :class="{ selected: selectedVault === vault.id }"
              @click="selectedVault = vault.id"
            >
              <span class="vault-icon">
                <FolderOpened :size="18" />
              </span>
              <div class="vault-info">
                <span class="vault-name">{{ vault.name }}</span>
                <span class="vault-path">{{ vault.path }}</span>
                <span v-if="vault.last_opened" class="vault-date">
                  {{ formatDate(vault.last_opened) }}
                </span>
              </div>
              <el-button type="primary" size="small" @click.stop="openExistingVault(vault.id)">
                Open
              </el-button>
            </div>
          </el-card>
        </div>

        <div v-if="otherVaults.length > 0" class="vault-section">
          <h2>其他仓库</h2>
          <el-card v-for="vault in otherVaults" :key="vault.id" class="vault-card vault-card--secondary">
            <div
              class="vault-item"
              :class="{ selected: selectedVault === vault.id }"
              @click="selectedVault = vault.id"
            >
              <span class="vault-icon">
                <FolderOpened :size="16" />
              </span>
              <div class="vault-info">
                <span class="vault-name">{{ vault.name }}</span>
                <span class="vault-path">{{ vault.path }}</span>
              </div>
              <el-button type="default" size="small" @click.stop="openExistingVault(vault.id)">
                Open
              </el-button>
            </div>
          </el-card>
        </div>
      </div>

      <div v-else class="empty-hint">
        <FolderOpened :size="32" class="empty-icon" />
        <p>还没有打开过仓库</p>
        <p class="empty-desc">点击上方按钮选择一个文件夹作为你的知识仓库</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.vault-picker {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--roc-bg-primary) 0%, var(--roc-bg-secondary) 100%);
}

.vault-picker-content {
  width: 100%;
  max-width: 560px;
  padding: 48px 32px;
}

.vault-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--roc-text-primary);
  margin: 0 0 8px 0;
}

.vault-subtitle {
  font-size: 14px;
  color: var(--roc-text-muted);
  margin: 0 0 32px 0;
}

.primary-btn {
  width: 100%;
  margin-bottom: 32px;
}

.vault-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.vault-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 6px;
}

.section-header h2 {
  font-size: 14px;
  font-weight: 600;
  color: var(--roc-text-secondary);
  margin: 0;
}

.section-header .el-icon {
  color: var(--roc-accent);
}

.vault-card {
  cursor: pointer;
  transition: all 0.2s;
}

.vault-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
}

.vault-card--secondary {
  opacity: 0.85;
}

.vault-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  transition: background 0.15s;
}

.vault-item:hover {
  background: var(--roc-bg-tertiary);
}

.vault-item.selected {
  background: rgba(0, 122, 204, 0.08);
}

.vault-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--roc-bg-tertiary);
  border-radius: 8px;
  color: var(--roc-accent);
}

.vault-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.vault-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--roc-text-primary);
}

.vault-path {
  font-size: 12px;
  color: var(--roc-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.vault-date {
  font-size: 11px;
  color: var(--roc-accent);
}

.empty-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 48px 24px;
  text-align: center;
}

.empty-icon {
  color: var(--roc-text-muted);
  opacity: 0.3;
  margin-bottom: 16px;
}

.empty-hint p {
  margin: 0;
  font-size: 14px;
  color: var(--roc-text-secondary);
}

.empty-desc {
  font-size: 12px;
  color: var(--roc-text-muted);
  margin-top: 6px !important;
}
</style>
