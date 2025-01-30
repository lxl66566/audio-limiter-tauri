import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createSignal, onMount } from "solid-js";

function App() {
  const [volume, setVolume] = createSignal(-20);
  const [isDarkMode, setIsDarkMode] = createSignal(false);

  onMount(async () => {
    // 检测系统暗黑模式
    const darkModeMediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    setIsDarkMode(darkModeMediaQuery.matches);

    // 监听系统暗黑模式变化
    darkModeMediaQuery.addEventListener("change", (e) => {
      setIsDarkMode(e.matches);
    });

    // 获取初始音量
    const initialVolume: number = await invoke("get_volume");
    setVolume(initialVolume);

    // 监听音量变化事件
    await listen("volume-changed", (event: any) => {
      setVolume(event.payload);
    });
  });

  const handleVolumeChange = async (event: Event) => {
    const value = parseInt((event.target as HTMLInputElement).value);
    setVolume(value);
    await invoke("set_volume", { value });
  };

  return (
    <div
      class={`flex flex-row items-center justify-center gap-2 h-screen w-screen ${isDarkMode() ? "bg-gray-900 text-white" : "bg-white text-black"}`}
      data-tauri-drag-region
    >
      <span></span>
      <input
        type="range"
        min="-50"
        max="0"
        value={volume()}
        onInput={handleVolumeChange}
        class={`flex-1 min-w-0 ${isDarkMode() ? "accent-blue-400" : "accent-blue-600"}`}
      />
      <span class="w-6 text-center text-sm">{volume()}</span>
      <span></span>
    </div>
  );
}

export default App;
