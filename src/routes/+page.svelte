<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import { invoke } from "@tauri-apps/api/core";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import WidgetRenderer from "./WidgetRenderer.svelte";

  // Mode: 'loading' (initial), 'dashboard' (editor/preview) or 'widget' (transparent widget window)
  let mode = $state("loading");
  let widgetConfig = $state<any>(null);
  let jsonInput = $state("");
  let parseError = $state("");
  let pinActive = $state(false);

  // Default Weather Widget Config for Suwon
  const weatherTemplate = {
    title: "수원 날씨 위젯",
    width: 300,
    height: 185,
    style: "background: rgba(15, 23, 42, 0.45); backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px); border-radius: 20px; border: 1px solid rgba(255, 255, 255, 0.15); padding: 18px; font-family: system-ui, sans-serif; color: white; box-shadow: 0 10px 30px rgba(0,0,0,0.3);",
    dataSources: [
      {
        id: "weather",
        url: "https://api.open-meteo.com/v1/forecast?latitude=37.2636&longitude=127.0286&current_weather=true",
        interval: 60000,
        mappings: {
          temp: "current_weather.temperature",
          wind: "current_weather.windspeed",
          code: "current_weather.weathercode"
        }
      }
    ],
    elements: [
      {
        type: "row",
        style: "justify-content: space-between; margin-bottom: 2px;",
        elements: [
          {
            type: "text",
            content: "수원 날씨",
            style: "font-size: 13px; font-weight: 700; color: rgba(255, 255, 255, 0.7); letter-spacing: 0.5px;"
          },
          {
            type: "text",
            content: "{time}",
            style: "font-size: 11px; font-weight: 500; color: rgba(255, 255, 255, 0.45);"
          }
        ]
      },
      {
        type: "row",
        style: "align-items: baseline; margin-bottom: 12px;",
        elements: [
          {
            type: "text",
            content: "{weather.icon}",
            style: "font-size: 34px; margin-right: 8px; line-height: 1;"
          },
          {
            type: "text",
            content: "{weather.temp}°C",
            style: "font-size: 30px; font-weight: 300; letter-spacing: -1px;"
          },
          {
            type: "text",
            content: "{weather.desc}",
            style: "font-size: 13px; font-weight: 600; color: #a5f3fc; margin-left: 10px;"
          }
        ]
      },
      {
        type: "row",
        style: "justify-content: space-between; margin-bottom: 5px;",
        elements: [
          {
            type: "text",
            content: "CPU 사용량",
            style: "font-size: 11px; color: rgba(255, 255, 255, 0.6);"
          },
          {
            type: "text",
            content: "{cpu}%",
            style: "font-size: 11px; font-weight: 700; color: #38bdf8;"
          }
        ]
      },
      {
        type: "progress",
        value: "{cpu}",
        style: "height: 4px; background: rgba(255,255,255,0.08); margin-bottom: 9px;",
        barStyle: "background: linear-gradient(90deg, #0ea5e9, #38bdf8);"
      },
      {
        type: "row",
        style: "justify-content: space-between; margin-bottom: 5px;",
        elements: [
          {
            type: "text",
            content: "RAM 사용량",
            style: "font-size: 11px; color: rgba(255, 255, 255, 0.6);"
          },
          {
            type: "text",
            content: "{ram}%",
            style: "font-size: 11px; font-weight: 700; color: #fb7185;"
          }
        ]
      },
      {
        type: "progress",
        value: "{ram}",
        style: "height: 4px; background: rgba(255,255,255,0.08);",
        barStyle: "background: linear-gradient(90deg, #f43f5e, #fb7185);"
      }
    ]
  };

  // Compact System Info Widget Config
  const systemTemplate = {
    title: "심플 모니터 위젯",
    width: 260,
    height: 115,
    style: "background: rgba(30, 41, 59, 0.7); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); border-radius: 16px; border: 1px solid rgba(255, 255, 255, 0.1); padding: 16px; font-family: system-ui, sans-serif; color: white;",
    elements: [
      {
        type: "row",
        style: "justify-content: space-between; margin-bottom: 10px;",
        elements: [
          {
            type: "text",
            content: "System Stat",
            style: "font-size: 12px; font-weight: 800; color: #94a3b8; letter-spacing: 1px;"
          },
          {
            type: "text",
            content: "{time}",
            style: "font-size: 12px; font-weight: 600; color: #38bdf8;"
          }
        ]
      },
      {
        type: "row",
        style: "align-items: center; margin-bottom: 8px;",
        elements: [
          {
            type: "text",
            content: "CPU",
            style: "font-size: 11px; font-weight: 600; width: 30px;"
          },
          {
            type: "progress",
            value: "{cpu}",
            style: "height: 6px; background: rgba(255,255,255,0.1); margin-right: 8px;",
            barStyle: "background: #10b981;"
          },
          {
            type: "text",
            content: "{cpu}%",
            style: "font-size: 11px; font-weight: 700; width: 35px; text-align: right;"
          }
        ]
      },
      {
        type: "row",
        style: "align-items: center;",
        elements: [
          {
            type: "text",
            content: "RAM",
            style: "font-size: 11px; font-weight: 600; width: 30px;"
          },
          {
            type: "progress",
            value: "{ram}",
            style: "height: 6px; background: rgba(255,255,255,0.1); margin-right: 8px;",
            barStyle: "background: #f59e0b;"
          },
          {
            type: "text",
            content: "{ram}%",
            style: "font-size: 11px; font-weight: 700; width: 35px; text-align: right;"
          }
        ]
      }
    ]
  };

  onMount(async () => {
    // Remove the fallback loading screen from app.html since Svelte has mounted successfully
    const fallbackEl = document.getElementById("fallback-loading-ui");
    if (fallbackEl) {
      fallbackEl.remove();
    }

    try {
      const windowLabel = getCurrentWindow().label;
      
      if (windowLabel.startsWith("widget-")) {
        mode = "widget";
        try {
          const configStr: string | null = await invoke("get_widget_config", { label: windowLabel });
          if (configStr) {
            widgetConfig = JSON.parse(configStr);
            
            const width = widgetConfig.width || 300;
            const height = widgetConfig.height || 200;
            try {
              await getCurrentWindow().setSize(new LogicalSize(width, height));
            } catch (sizeErr: any) {
              console.error(`Failed to set window size: ${sizeErr.message || sizeErr}`);
            }
          } else {
            parseError = "위젯 설정을 찾을 수 없습니다 (Rust State Empty).";
          }
        } catch (e: any) {
          parseError = "위젯 데이터 로딩 에러: " + e.message;
        }
      } else {
        mode = "dashboard";
        loadTemplate(weatherTemplate);
        
        // Auto-spawn widgets from localStorage
        const saved = localStorage.getItem("snowmeter_widgets");
        const activeWidgets = saved ? JSON.parse(saved) : [];
        
        if (activeWidgets.length > 0) {
          activeWidgets.forEach((w: any) => {
            spawnWidget(w.id, w.config);
          });
        } else {
          // If no widgets are running, automatically show the dashboard
          getCurrentWindow().show();
          getCurrentWindow().setFocus();
        }

        try {
          await getCurrentWindow().setSize(new LogicalSize(850, 620));
        } catch (sizeErr) {
          console.error("Failed to restore dashboard window size:", sizeErr);
        }
      }
    } catch (err: any) {
      parseError = "초기화 에러: " + err.message;
    }
  });

  function loadTemplate(tpl: any) {
    jsonInput = JSON.stringify(tpl, null, 2);
    handleJsonInput();
  }

  function handleJsonInput() {
    try {
      widgetConfig = JSON.parse(jsonInput);
      parseError = "";
    } catch (e: any) {
      parseError = "JSON 형식 오류: " + e.message;
    }
  }

  async function launchWidget() {
    if (parseError || !widgetConfig) return;
    try {
      const windowLabel = `widget-${Date.now()}`;
      
      // Save to localStorage
      const saved = localStorage.getItem("snowmeter_widgets");
      let activeWidgets = saved ? JSON.parse(saved) : [];
      activeWidgets.push({ id: windowLabel, config: widgetConfig });
      localStorage.setItem("snowmeter_widgets", JSON.stringify(activeWidgets));

      spawnWidget(windowLabel, widgetConfig);
    } catch (e: any) {
      alert("위젯 실행 오류: " + e);
    }
  }

  async function spawnWidget(windowLabel: string, config: any) {
    // 1. Save config to Rust state so the new window can fetch it
    await invoke("save_widget_config", {
      label: windowLabel,
      config: JSON.stringify(config)
    });

    // 2. Create the window dynamically via JS
    const width = config.width || 300;
    const height = config.height || 200;
    
    new WebviewWindow(windowLabel, {
      url: "/", 
      title: windowLabel,
      width: width,
      height: height,
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      shadow: false,
      skipTaskbar: true
    });
  }

  // Window Dragging
  async function handleDrag(e: MouseEvent) {
    // Only drag on left click and if they aren't clicking a button
    if (e.button === 0 && (e.target as HTMLElement).tagName !== 'BUTTON') {
      try {
        await getCurrentWindow().startDragging();
      } catch (err) {
        console.error("Failed to start dragging:", err);
      }
    }
  }

  // Dashboard Window Actions
  async function onPinClick() {
    pinActive = !pinActive;
    await invoke("toggle_always_on_top", { window: getCurrentWindow() });
  }

  function onMinimizeClick() {
    getCurrentWindow().minimize();
  }

  function onMaximizeClick() {
    getCurrentWindow().toggleMaximize();
  }

  async function onCloseClick() {
    if (mode === "widget") {
      const windowLabel = getCurrentWindow().label;
      const saved = localStorage.getItem("snowmeter_widgets");
      if (saved) {
        let activeWidgets = JSON.parse(saved);
        activeWidgets = activeWidgets.filter((w: any) => w.id !== windowLabel);
        localStorage.setItem("snowmeter_widgets", JSON.stringify(activeWidgets));
      }
    } else {
      // If closing dashboard, don't kill the app, just hide it so widgets stay alive and tray icon remains
      await getCurrentWindow().hide();
      return;
    }
    
    await getCurrentWindow().close();
  }
</script>

{#if mode === 'dashboard'}
  <!-- DASHBOARD UI -->
  <main class="dashboard-container">
    <!-- Beautiful frameless window title bar -->
    <div data-tauri-drag-region class="titlebar">
      <span class="titlebar-logo">❄️ SNOWMETER - Widget Center</span>
      <div class="titlebar-actions">
        <button class="titlebar-btn" class:active={pinActive} onclick={onPinClick} title="Always on top">
          📌
        </button>
        <button class="titlebar-btn" onclick={onMinimizeClick} title="Minimize">
          ─
        </button>
        <button class="titlebar-btn" onclick={onMaximizeClick} title="Maximize">
          ⛶
        </button>
        <button class="titlebar-btn close-btn" onclick={onCloseClick} title="Close">
          ✕
        </button>
      </div>
    </div>

    <!-- Dashboard Content Layout -->
    <div class="dashboard-body">
      <!-- Left Panel: JSON Editor & Template Selectors -->
      <section class="editor-section">
        <div class="section-header">
          <h3>JSON 위젯 템플릿</h3>
          <div class="template-selector">
            <button onclick={() => loadTemplate(weatherTemplate)} class="template-btn">수원 날씨 위젯</button>
            <button onclick={() => loadTemplate(systemTemplate)} class="template-btn">심플 시스템 위젯</button>
          </div>
        </div>

        <textarea 
          bind:value={jsonInput} 
          oninput={handleJsonInput}
          placeholder="위젯 JSON 명세를 작성해주세요..." 
          class="json-textarea"
          spellcheck="false"
        ></textarea>

        {#if parseError}
          <div class="error-banner">{parseError}</div>
        {/if}

        <button 
          onclick={launchWidget} 
          disabled={!!parseError} 
          class="launch-action-btn"
        >
          🚀 바탕화면에 독립 위젯으로 띄우기
        </button>
      </section>

      <!-- Right Panel: Live Rendering Preview -->
      <section class="preview-section">
        <div class="section-header">
          <h3>실시간 미리보기 (Live Preview)</h3>
        </div>
        <div class="preview-canvas">
          {#if widgetConfig && !parseError}
            <WidgetRenderer config={widgetConfig} />
          {:else}
            <div class="preview-placeholder">
              <span class="placeholder-icon">⚙️</span>
              <p>올바른 JSON 구문을 입력하시면<br>여기에 실시간 위젯이 렌더링됩니다.</p>
            </div>
          {/if}
        </div>
      </section>
    </div>
  </main>
{:else if mode === 'widget'}
  <!-- TRANSPARENT WIDGET WINDOW MODE -->
  <main class="widget-window-container" onmousedown={handleDrag}>
    <!-- Close Widget button (visible on hover) -->
    <button class="widget-close-action" onclick={onCloseClick} title="위젯 닫기">
      ✕
    </button>
    {#if widgetConfig}
      <WidgetRenderer config={widgetConfig} />
    {:else if parseError}
      <div class="widget-error">{parseError}</div>
    {:else}
      <!-- Loading spinner while fetching config from Rust (usually instant) -->
      <div class="widget-loading-wrapper" data-tauri-drag-region>
        <div class="widget-loading-spinner" data-tauri-drag-region></div>
      </div>
    {/if}
  </main>
{:else}
  <!-- LOADING MODE (keeps window fully transparent and clean on launch) -->
  <div class="window-loading-placeholder" data-tauri-drag-region>
    {#if parseError}
      <div style="background: red; color: white; padding: 20px; border-radius: 8px; margin: 10px;">
        <h2>FATAL ERROR</h2>
        <p>{parseError}</p>
      </div>
    {/if}
  </div>
{/if}

<style lang="scss">
  // Global resets to support full transparency & beautiful typography
  :global(html), :global(body) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: transparent !important;
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
  }

  // Common UI Styles
  button {
    font-family: inherit;
  }

  /* ----------------- DASHBOARD MODE ----------------- */
  .dashboard-container {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background-color: #0f172a;
    color: #f8fafc;
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-sizing: border-box;
    overflow: hidden;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }

  .titlebar {
    height: 40px;
    background-color: #1e293b;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px;
    user-select: none;
    cursor: move;

    .titlebar-logo {
      font-size: 13px;
      font-weight: 700;
      color: #38bdf8;
      letter-spacing: 0.5px;
    }

    .titlebar-actions {
      display: flex;
      gap: 6px;

      .titlebar-btn {
        background: transparent;
        border: none;
        color: #94a3b8;
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 6px;
        cursor: pointer;
        font-size: 12px;
        transition: all 0.2s ease;

        &:hover {
          background-color: rgba(255, 255, 255, 0.08);
          color: #f8fafc;
        }

        &.active {
          background-color: #0ea5e9;
          color: white;
        }

        &.close-btn:hover {
          background-color: #ef4444;
          color: white;
        }
      }
    }
  }

  .dashboard-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .editor-section, .preview-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 20px;
    overflow: hidden;
  }

  .editor-section {
    border-right: 1px solid rgba(255, 255, 255, 0.05);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;

    h3 {
      margin: 0;
      font-size: 15px;
      font-weight: 600;
      color: #e2e8f0;
    }

    .template-selector {
      display: flex;
      gap: 6px;

      .template-btn {
        background-color: #1e293b;
        color: #94a3b8;
        border: 1px solid rgba(255,255,255,0.05);
        padding: 5px 10px;
        font-size: 11px;
        font-weight: 600;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s ease;

        &:hover {
          background-color: #334155;
          color: #f8fafc;
          border-color: rgba(255,255,255,0.1);
        }
      }
    }
  }

  .json-textarea {
    flex: 1;
    background-color: #020617;
    color: #38bdf8;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    padding: 16px;
    font-family: 'Courier New', Courier, monospace;
    font-size: 13px;
    line-height: 1.5;
    resize: none;
    outline: none;
    box-sizing: border-box;

    &:focus {
      border-color: #0ea5e9;
      box-shadow: 0 0 0 1px rgba(14, 165, 233, 0.3);
    }
  }

  .error-banner {
    background-color: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.25);
    border-radius: 8px;
    padding: 10px 14px;
    font-size: 12px;
    margin-top: 10px;
  }

  .launch-action-btn {
    background: linear-gradient(135deg, #0284c7, #0ea5e9);
    color: white;
    border: none;
    border-radius: 12px;
    padding: 14px 20px;
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
    margin-top: 14px;
    box-shadow: 0 4px 12px rgba(14, 165, 233, 0.3);
    transition: all 0.2s ease;

    &:hover:not(:disabled) {
      transform: translateY(-1px);
      box-shadow: 0 6px 16px rgba(14, 165, 233, 0.45);
    }

    &:active:not(:disabled) {
      transform: translateY(0);
    }

    &:disabled {
      background: #334155;
      color: #64748b;
      cursor: not-allowed;
      box-shadow: none;
    }
  }

  .preview-section {
    align-items: center;
    justify-content: flex-start;
  }

  .preview-canvas {
    flex: 1;
    width: 100%;
    background-color: #020617;
    background-image: radial-gradient(rgba(255,255,255,0.02) 1.5px, transparent 1.5px);
    background-size: 16px 16px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: auto;
  }

  .preview-placeholder {
    text-align: center;
    color: #64748b;

    .placeholder-icon {
      font-size: 40px;
      display: block;
      margin-bottom: 12px;
      animation: spin 8s linear infinite;
    }

    p {
      margin: 0;
      font-size: 13px;
      line-height: 1.6;
    }
  }

  /* ----------------- TRANSPARENT WIDGET WINDOW MODE ----------------- */
  .widget-window-container {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    align-items: flex-start;
    justify-content: flex-start;
    position: relative;
    background: transparent;
    cursor: move; // Clicking anywhere on the widget window moves it

    &:hover .widget-close-action {
      opacity: 1;
    }
  }

  .widget-close-action {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background-color: rgba(0, 0, 0, 0.5);
    color: rgba(255, 255, 255, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.1);
    font-size: 9px;
    font-weight: bold;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0;
    transition: all 0.2s ease;
    z-index: 1000;

    &:hover {
      background-color: #ef4444;
      color: white;
      border-color: transparent;
    }
  }

  .widget-error {
    background: rgba(239, 68, 68, 0.85);
    color: white;
    padding: 16px;
    border-radius: 12px;
    font-size: 13px;
    font-weight: 500;
  }

  .window-loading-placeholder {
    width: 100%;
    height: 100%;
    background: transparent;
  }

  .widget-loading-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    background: rgba(15, 23, 42, 0.45);
    backdrop-filter: blur(8px);
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .widget-loading-spinner {
    width: 24px;
    height: 24px;
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-top-color: #38bdf8;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }


  /* Animations */
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
