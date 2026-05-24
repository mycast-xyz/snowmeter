<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // Svelte 5 Props
  let { config } = $props<{ config: any }>();

  // State definitions
  let systemStats = $state({ cpu: "0.0", ram: "0.0" });
  let currentTime = $state(new Date());
  let apiData = $state<Record<string, any>>({});

  // WMO Weather Code Mapper (for Open-Meteo)
  function getWeatherDescription(code: number): { text: string; icon: string } {
    switch (code) {
      case 0: return { text: "맑음", icon: "☀️" };
      case 1: return { text: "대체로 맑음", icon: "🌤️" };
      case 2: return { text: "구름 조금", icon: "⛅" };
      case 3: return { text: "흐림", icon: "☁️" };
      case 45: case 48: return { text: "안개", icon: "🌫️" };
      case 51: case 53: case 55: return { text: "가랑비", icon: "🌧️" };
      case 61: case 63: case 65: return { text: "비", icon: "🌧️" };
      case 71: case 73: case 75: return { text: "눈", icon: "❄️" };
      case 77: return { text: "싸락눈", icon: "❄️" };
      case 80: case 81: case 82: return { text: "소나기", icon: "🌧️" };
      case 85: case 86: return { text: "소나기 눈", icon: "❄️" };
      case 95: case 96: case 99: return { text: "뇌우", icon: "⛈️" };
      default: return { text: "알 수 없음", icon: "❓" };
    }
  }

  // Path resolution helper (e.g. "current_weather.temperature")
  function getValueByPath(obj: any, path: string): any {
    return path.split(".").reduce((acc, part) => acc && acc[part], obj);
  }

  // Derive all active state values for placeholder substitution
  let state = $derived.by(() => {
    const result: Record<string, any> = {
      cpu: systemStats.cpu,
      ram: systemStats.ram,
      time: currentTime.toLocaleTimeString("ko-KR", { hour12: false }),
      date: currentTime.toLocaleDateString("ko-KR"),
      hour: String(currentTime.getHours()).padStart(2, "0"),
      minute: String(currentTime.getMinutes()).padStart(2, "0"),
      second: String(currentTime.getSeconds()).padStart(2, "0"),
    };

    // Populate API Data source variables
    for (const [dsId, dsData] of Object.entries(apiData)) {
      for (const [key, val] of Object.entries(dsData)) {
        result[`${dsId}.${key}`] = val;
      }
    }

    return result;
  });

  // Check if any dataSources are currently loading
  let isLoading = $derived.by(() => {
    if (!config?.dataSources || !Array.isArray(config.dataSources)) return false;
    for (const ds of config.dataSources) {
      if (!apiData[ds.id]) return true;
    }
    return false;
  });

  // Safe placeholder replacement
  function replacePlaceholders(template: string): string {
    if (typeof template !== "string") return template;
    return template.replace(/\{([^}]+)\}/g, (match, key) => {
      return state[key] !== undefined ? String(state[key]) : match;
    });
  }

  // Resolve numerical values for progress bars
  function resolveValue(val: any): number {
    if (typeof val === "string") {
      const resolved = replacePlaceholders(val);
      const num = parseFloat(resolved);
      return isNaN(num) ? 0 : num;
    }
    return typeof val === "number" ? val : 0;
  }

  // Fetch API data helper
  async function fetchDataSource(ds: any) {
    try {
      const res = await fetch(ds.url);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      
      const resolved: Record<string, any> = {};
      for (const [key, jsonPath] of Object.entries(ds.mappings || {})) {
        const val = getValueByPath(data, jsonPath as string);
        resolved[key] = val;

        // Auto-decode weather codes if key is 'code'
        if (key === "code") {
          const descInfo = getWeatherDescription(Number(val));
          resolved["desc"] = descInfo.text;
          resolved["icon"] = descInfo.icon;
        }
      }

      apiData[ds.id] = resolved;
    } catch (err: any) {
      console.error(`Error fetching datasource '${ds.id}':`, err);
      // Save error details so we don't get stuck in a loading state
      apiData[ds.id] = { error: true, message: err.message || "네트워크 에러" };
    }
  }

  // Lifecycle effects
  onMount(() => {
    // 1. Fetch system CPU/RAM stats
    const statsInterval = setInterval(async () => {
      try {
        const stats: any = await invoke("get_system_stats");
        systemStats.cpu = stats.cpu_usage;
        systemStats.ram = stats.ram_usage;
      } catch (e) {
        console.error("Failed to fetch system stats:", e);
      }
    }, 1000);

    // 2. Clock timer
    const clockInterval = setInterval(() => {
      currentTime = new Date();
    }, 1000);

    // 3. API Data source loaders
    const apiIntervals: number[] = [];
    if (config?.dataSources && Array.isArray(config.dataSources)) {
      for (const ds of config.dataSources) {
        // Initial fetch
        fetchDataSource(ds);
        
        // Interval fetch (default to 5 minutes if not specified)
        const intervalMs = ds.interval || 300000;
        const interval = setInterval(() => {
          fetchDataSource(ds);
        }, intervalMs);
        apiIntervals.push(interval);
      }
    }

    return () => {
      clearInterval(statsInterval);
      clearInterval(clockInterval);
      for (const inv of apiIntervals) {
        clearInterval(inv);
      }
    };
  });
</script>

{#if config}
  <div 
    class="widget-container" 
    style="{config.style || ''}; width: {config.width ? config.width + 'px' : 'auto'}; height: {config.height ? config.height + 'px' : 'auto'};"
  >
    {#if isLoading}
      <!-- Loading indicator (fully draggable) -->
      <div class="loading-wrapper" data-tauri-drag-region>
        <div class="spinner" data-tauri-drag-region></div>
        <div class="loading-text" data-tauri-drag-region>날씨 정보 수신 중...</div>
      </div>
    {:else}
      {#each Object.entries(apiData) as [dsId, data]}
        {#if data.error}
          <div class="datasource-error-banner" data-tauri-drag-region>
            ⚠️ '{dsId}' 갱신 실패 ({data.message})
          </div>
        {/if}
      {/each}
      {#if config.elements && Array.isArray(config.elements)}
        {#each config.elements as el}
          <!-- Render text element -->
          {#if el.type === 'text'}
            <div style={el.style || ''}>
              {replacePlaceholders(el.content)}
            </div>

          <!-- Render progress bar element -->
          {:else if el.type === 'progress'}
            <div class="progress-track" style={el.style || ''}>
              <div 
                class="progress-bar" 
                style="{el.barStyle || ''}; width: {Math.min(100, Math.max(0, resolveValue(el.value)))}%;"
              ></div>
            </div>

          <!-- Render row alignment -->
          {:else if el.type === 'row'}
            <div class="flex-row" style={el.style || ''}>
              {#if el.elements && Array.isArray(el.elements)}
                {#each el.elements as subEl}
                  {#if subEl.type === 'text'}
                    <span style={subEl.style || ''}>
                      {replacePlaceholders(subEl.content)}
                    </span>
                  {:else if subEl.type === 'progress'}
                    <div class="progress-track" style="flex: 1; {subEl.style || ''}">
                      <div 
                        class="progress-bar" 
                        style="{subEl.barStyle || ''}; width: {Math.min(100, Math.max(0, resolveValue(subEl.value)))}%;"
                      ></div>
                    </div>
                  {/if}
                {/each}
              {/if}
            </div>

          <!-- Render spacer -->
          {:else if el.type === 'spacer'}
            <div style="height: {el.height || '8px'};"></div>
          {/if}
        {/each}
      {/if}
    {/if}
  </div>
{:else}
  <div class="error-msg">설정을 로드하지 못했습니다.</div>
{/if}

<style>
  .widget-container {
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
  }

  .datasource-error-banner {
    background-color: rgba(239, 68, 68, 0.2);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.4);
    font-size: 11px;
    padding: 6px 10px;
    border-radius: 8px;
    margin-bottom: 8px;
    text-align: center;
    cursor: move;
  }

  .loading-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    min-height: 100px;
    gap: 12px;
    user-select: none;
    cursor: move;
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 3px solid rgba(255, 255, 255, 0.15);
    border-top: 3px solid #38bdf8;
    border-radius: 50%;
    animation: loading-spin 1s linear infinite;
  }

  .loading-text {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.7);
    font-weight: 500;
    letter-spacing: 0.5px;
  }

  .flex-row {
    display: flex;
    align-items: center;
    width: 100%;
  }

  .progress-track {
    box-sizing: border-box;
    position: relative;
    background-color: rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    height: 8px;
    width: 100%;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    border-radius: 4px;
    background-color: #3b82f6;
    transition: width 0.3s ease-out;
  }

  .error-msg {
    color: #ef4444;
    padding: 10px;
    font-size: 14px;
  }

  @keyframes loading-spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
