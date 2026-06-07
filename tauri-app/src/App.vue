<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watchEffect } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useStorage } from '@vueuse/core';

// Icons
import { Mic, Wifi, RadioTower, Globe, ChevronDown, CheckCircle2, Play, Square, Settings, Puzzle } from 'lucide-vue-next';
import CustomBackground from './components/CustomBackground.vue';
import SettingsDialog from './components/SettingsDialog.vue';
import AudioRing from './components/AudioRing.vue';

const serverState = ref<'idle' | 'connecting' | 'streaming'>('idle');
const connectionMode = ref<'wifi' | 'usb' | 'web'>('wifi');
const serverPort = ref(9123);
const audioLevel = ref(0);
const networkInfo = ref<{ ips: string[], port: number } | null>(null);
const selectedIp = ref<string>('0.0.0.0');

const isSettingsOpen = ref(false);
const outputDevice = ref<string>(localStorage.getItem('micyou_output_device') || '');

// Global Theme and UI Style Management
const themeColor = useStorage('micyou_theme_color', 'theme-blue');
const uiStyle = useStorage('micyou_ui_style', 'style-glass');

watchEffect(() => {
  if (typeof document !== 'undefined') {
    document.documentElement.classList.remove('theme-blue', 'theme-green', 'theme-rose', 'theme-purple');
    document.documentElement.classList.remove('style-default', 'style-glass');
    
    if (themeColor.value) document.documentElement.classList.add(themeColor.value);
    if (uiStyle.value) document.documentElement.classList.add(uiStyle.value);
  }
});



let unlistenAudioLevel: UnlistenFn | null = null;
let unlistenDeviceConnected: UnlistenFn | null = null;
let unlistenDeviceDisconnected: UnlistenFn | null = null;

onMounted(async () => {

  try {
    networkInfo.value = await invoke<{ ips: string[], port: number }>('get_network_info');
    if (networkInfo.value && networkInfo.value.ips.length > 0) {
      selectedIp.value = networkInfo.value.ips[0];
    }
  } catch (e) {
    console.error("Failed to get network info:", e);
  }

  unlistenAudioLevel = await listen<number>('audio-level', (event) => {
    audioLevel.value = event.payload;
  });

  unlistenDeviceConnected = await listen('device-connected', () => {
    serverState.value = 'streaming';
  });

  unlistenDeviceDisconnected = await listen('device-disconnected', () => {
    if (serverState.value === 'streaming') {
      serverState.value = 'connecting'; // Go back to waiting for device
      audioLevel.value = 0;
    }
  });
});

onUnmounted(() => {
  if (unlistenAudioLevel) unlistenAudioLevel();
  if (unlistenDeviceConnected) unlistenDeviceConnected();
  if (unlistenDeviceDisconnected) unlistenDeviceDisconnected();
});

const toggleStreaming = async () => {
  if (serverState.value !== 'idle') {
    try {
      await invoke('stop_server');
      serverState.value = 'idle';
      audioLevel.value = 0;
    } catch (e) {
      console.error(e);
    }
  } else {
    try {
      await invoke('start_server', { 
        port: Number(serverPort.value), 
        mode: connectionMode.value,
        outputDevice: outputDevice.value || null
      });
      serverState.value = 'connecting';
      if (connectionMode.value === 'usb') {
        await invoke('enable_usb_mode', { port: Number(serverPort.value) });
      }
    } catch (e) {
      console.error(e);
    }
  }
};

const micScale = computed(() => {
  return 1 + (audioLevel.value / 100) * 0.5;
});
</script>

<template>
  <div class="relative w-full h-screen overflow-hidden text-foreground">
    <CustomBackground />
    
    <div class="absolute inset-0 flex flex-col p-3 gap-3">
      <!-- Header Section -->
      <div class="haze-surface rounded-2xl flex justify-between items-center px-4 py-2 flex-shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-primary-container flex items-center justify-center shadow-sm border border-primary/20">
            <RadioTower class="w-5 h-5 text-primary" />
          </div>
          <div class="flex flex-col">
            <span class="text-sm font-extrabold text-primary">MicYou Desktop</span>
            <span class="text-[11px] text-on-surface-variant font-medium">Server</span>
          </div>
        </div>

        <div class="flex items-center bg-surface-variant/30 hover:bg-surface-variant/50 transition-colors px-3 py-1.5 rounded-lg cursor-pointer border border-white/5">
          <Globe class="w-3.5 h-3.5 text-primary mr-2" />
          <span class="text-xs font-medium mr-1 select-none">{{ selectedIp === '0.0.0.0' ? 'All Interfaces' : selectedIp }}</span>
          <ChevronDown class="w-4 h-4 text-on-surface-variant/60" />
        </div>
      </div>

      <!-- Main Content -->
      <div class="flex flex-1 gap-3 min-h-0">
        <!-- Left Panel (38%) -->
        <div class="w-[38%] flex flex-col gap-3">
          <!-- Mode Card -->
          <div class="haze-surface rounded-2xl p-3 flex flex-col gap-2">
            <span class="text-xs text-on-surface-variant font-medium">{{ $t('app.connectionMode') }}</span>
            <div class="flex gap-1.5">
              <button 
                @click="connectionMode = 'wifi'"
                class="flex-1 flex flex-col items-center justify-center py-2 rounded-xl transition-colors duration-200"
                :class="connectionMode === 'wifi' ? 'bg-primary text-on-primary' : 'bg-surface-variant/40 text-on-surface-variant hover:bg-surface-variant/60'"
              >
                <Wifi class="w-4 h-4 mb-1" />
                <span class="text-[10px] font-medium">Wi-Fi</span>
              </button>
              <button 
                @click="connectionMode = 'usb'"
                class="flex-1 flex flex-col items-center justify-center py-2 rounded-xl transition-colors duration-200"
                :class="connectionMode === 'usb' ? 'bg-primary text-on-primary' : 'bg-surface-variant/40 text-on-surface-variant hover:bg-surface-variant/60'"
              >
                <Mic class="w-4 h-4 mb-1" />
                <span class="text-[10px] font-medium">USB</span>
              </button>
            </div>
          </div>

          <!-- Port Card -->
          <div class="haze-surface rounded-2xl p-3 flex flex-col gap-2">
            <span class="text-xs text-on-surface-variant font-medium">{{ $t('app.port') }}</span>
            <input 
              v-model="serverPort"
              type="number" 
              class="w-full bg-surface-variant/40 border border-white/5 rounded-xl px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-1 focus:ring-primary transition-all"
            />
          </div>

          <!-- Status Card -->
          <div class="haze-surface rounded-2xl p-4 flex-1 flex flex-col items-center justify-center text-center gap-3">
            <div class="w-12 h-12 rounded-full flex items-center justify-center transition-colors duration-500" 
                 :class="serverState === 'streaming' ? 'bg-primary/20 text-primary' : (serverState === 'connecting' ? 'bg-secondary/20 text-secondary' : 'bg-surface-variant/50 text-on-surface-variant')">
              <CheckCircle2 v-if="serverState === 'streaming'" class="w-6 h-6 animate-pulse" />
              <RadioTower v-else class="w-6 h-6" :class="{ 'animate-ping': serverState === 'connecting' }" />
            </div>
            <div>
              <h3 class="text-sm font-bold">{{ serverState === 'streaming' ? $t('app.status.streaming') : (serverState === 'connecting' ? $t('app.status.connecting') : $t('app.status.ready')) }}</h3>
              <p class="text-xs text-on-surface-variant mt-1 max-w-[200px]">
                {{ serverState === 'streaming' ? $t('app.status.streamingDesc') : (serverState === 'connecting' ? $t('app.status.connectingDesc', { port: serverPort }) : $t('app.status.readyDesc')) }}
              </p>
            </div>
          </div>
        </div>

        <!-- Center Panel (62%) -->
        <div class="w-[62%] haze-surface rounded-2xl flex items-center justify-center relative overflow-hidden">
          <!-- Central Mic with AudioRing -->
          <div class="relative w-64 h-64 flex items-center justify-center transition-transform duration-200"
               :style="{ transform: `scale(${serverState === 'streaming' ? micScale : 1})` }">
            <AudioRing :level="audioLevel">
              <Mic class="w-10 h-10 text-on-primary" />
            </AudioRing>
          </div>
          
          <div class="absolute bottom-6 font-mono text-sm text-primary/80 bg-primary/10 px-3 py-1 rounded-full border border-primary/20">
            {{ $t('app.level') }} {{ audioLevel }}
          </div>
        </div>
      </div>

      <!-- Bottom Bar -->
      <div class="haze-surface rounded-2xl p-2 flex justify-between items-center flex-shrink-0">
        <div class="flex items-center px-3">
          <div class="w-2 h-2 rounded-full mr-2" :class="serverState === 'streaming' ? 'bg-green-500 animate-pulse shadow-[0_0_8px_#22c55e]' : (serverState === 'connecting' ? 'bg-yellow-500 animate-pulse shadow-[0_0_8px_#eab308]' : 'bg-on-surface-variant')"></div>
          <span class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">{{ serverState === 'streaming' ? $t('app.status.stateStreaming') : (serverState === 'connecting' ? $t('app.status.stateConnecting') : $t('app.status.stateIdle')) }}</span>
        </div>
        
        <div class="flex items-center gap-2 pr-1">
          <button class="w-10 h-10 rounded-full bg-surface-variant/40 hover:bg-surface-variant/80 flex items-center justify-center transition-colors">
            <Puzzle class="w-4 h-4 text-on-surface-variant" />
          </button>
          <button @click="isSettingsOpen = true" class="w-10 h-10 rounded-full bg-surface-variant/40 hover:bg-surface-variant/80 flex items-center justify-center transition-colors">
            <Settings class="w-4 h-4 text-on-surface-variant" />
          </button>
          
          <button 
            @click="toggleStreaming"
            class="ml-2 px-6 h-10 rounded-full font-bold text-sm shadow-md transition-all duration-300 flex items-center gap-2"
            :class="serverState !== 'idle' ? 'bg-red-500 hover:bg-red-600 text-white shadow-red-500/20' : 'bg-primary hover:bg-primary/90 text-on-primary shadow-primary/20'"
          >
            <Square v-if="serverState !== 'idle'" class="w-4 h-4" />
            <Play v-else class="w-4 h-4" />
            {{ serverState !== 'idle' ? $t('app.stop') : $t('app.start') }}
          </button>
        </div>
      </div>
    </div>
    
    <SettingsDialog 
      :isOpen="isSettingsOpen" 
      @close="isSettingsOpen = false" 
      @updateDevice="dev => outputDevice = dev" 
    />
  </div>
</template>