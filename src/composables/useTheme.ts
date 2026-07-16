import { ref, onMounted, onUnmounted, watch } from 'vue';

type ThemeMode = 'light' | 'dark' | 'auto';

export function useTheme() {
  // 状态：当前主题模式
  const themeMode = ref<ThemeMode>((localStorage.getItem('theme-mode') as ThemeMode) || 'light');
  // 状态：实际生效的主题（由'mode'和系统主题计算得出）
  const activeTheme = ref<'light' | 'dark'>('light');

  // 判断系统是否开启暗色模式
  const getSystemTheme = (): 'light' | 'dark' => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  };

  // 应用主题到DOM的核心副作用
  const applyTheme = (theme: 'light' | 'dark') => {
    const htmlEl = document.documentElement;
    if (theme === 'dark') {
      htmlEl.classList.add('dark');
    } else {
      htmlEl.classList.remove('dark');
    }
    activeTheme.value = theme;
  };

  // 监听系统主题变化的MediaQueryList
  let mediaQuery: MediaQueryList;

  // 根据themeMode计算并应用主题
  const updateThemeFromMode = () => {
    if (themeMode.value === 'auto') {
      const systemTheme = getSystemTheme();
      applyTheme(systemTheme);
      // 监听系统变化
      if (!mediaQuery) {
        mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        mediaQuery.addEventListener('change', handleSystemThemeChange);
      }
    } else {
      applyTheme(themeMode.value);
      // 清除监听
      if (mediaQuery) {
        mediaQuery.removeEventListener('change', handleSystemThemeChange);
        mediaQuery = null!;
      }
    }
    // 持久化模式选择
    localStorage.setItem('theme-mode', themeMode.value);
  };

  const handleSystemThemeChange = (e: MediaQueryListEvent) => {
    if (themeMode.value === 'auto') {
      applyTheme(e.matches ? 'dark' : 'light');
    }
  };

  // 切换主题模式的方法
  const setThemeMode = (mode: ThemeMode) => {
    themeMode.value = mode;
    // watch会触发updateThemeFromMode
  };

  // 初始化 & 响应式监听
  onMounted(() => {
    updateThemeFromMode();
  });

  watch(themeMode, () => {
    updateThemeFromMode();
  });

  onUnmounted(() => {
    if (mediaQuery) {
      mediaQuery.removeEventListener('change', handleSystemThemeChange);
    }
  });

  return {
    themeMode,
    activeTheme,
    setThemeMode,
  };
}
