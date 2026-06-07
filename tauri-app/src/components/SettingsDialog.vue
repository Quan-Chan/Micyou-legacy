<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-8 bg-black/40 backdrop-blur-sm">
    <div class="bg-surface-bright w-full max-w-5xl h-full max-h-[80vh] rounded-3xl flex overflow-hidden shadow-2xl relative border border-white/10">
      
      <!-- Close Button -->
      <button @click="$emit('close')" class="absolute top-4 right-4 z-10 w-10 h-10 rounded-full bg-surface-variant/40 hover:bg-surface-variant/80 flex items-center justify-center transition-colors">
        <X class="w-5 h-5 text-on-surface" />
      </button>

      <!-- Left Sidebar -->
      <div class="w-64 bg-surface-container-low border-r border-surface-variant/30 flex flex-col p-4 space-y-2 overflow-y-auto">
        <div class="px-4 py-4 mb-4 flex items-center gap-3">
          <SettingsIcon class="w-6 h-6 text-primary" />
          <h2 class="text-xl font-bold text-primary">Settings</h2>
        </div>

        <button v-for="section in sections" :key="section.id" 
                @click="currentSection = section.id"
                class="flex items-center gap-3 px-4 py-3 rounded-2xl transition-all duration-300 w-full text-left"
                :class="currentSection === section.id ? 'bg-secondary-container text-on-secondary-container shadow-sm scale-[1.02]' : 'hover:bg-surface-variant/30 text-on-surface-variant'">
          <component :is="section.icon" class="w-5 h-5" :class="currentSection === section.id ? 'text-primary' : ''" />
          <span class="font-medium text-sm">{{ section.name }}</span>
        </button>
      </div>

      <!-- Right Content -->
      <div class="flex-1 bg-surface-container-lowest p-8 overflow-y-auto">
        <div class="max-w-2xl mx-auto space-y-8">
          <h3 class="text-3xl font-bold text-primary mb-6">{{ currentSectionName }}</h3>

          <!-- GENERAL SECTION -->
          <div v-if="currentSection === 'general'" class="space-y-6">
            <div class="bg-surface-bright rounded-2xl p-4 flex items-center justify-between shadow-sm">
              <div>
                <h4 class="font-bold text-on-surface">Language</h4>
                <p class="text-xs text-on-surface-variant">Choose application language</p>
              </div>
              <select class="bg-surface-container rounded-lg px-3 py-2 text-sm border-none outline-none">
                <option>English</option>
                <option>简体中文</option>
              </select>
            </div>

            <!-- VB-CABLE Management -->
            <div class="bg-surface-bright rounded-2xl p-4 space-y-4 shadow-sm">
              <div class="flex items-center justify-between">
                <h4 class="font-bold text-on-surface text-lg">VB-Cable Forwarding</h4>
                <span class="text-xs font-medium px-2 py-1 rounded-md" :class="hasVBCable ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'">
                  {{ hasVBCable ? 'Installed' : 'Not Detected' }}
                </span>
              </div>
              <p class="text-xs text-on-surface-variant">
                To route audio to other applications (like OBS or Discord), you need to install VB-Audio Virtual Cable. 
              </p>
              <button v-if="!hasVBCable" class="w-full py-2 bg-surface-variant hover:bg-surface-variant/80 rounded-xl text-sm font-bold flex items-center justify-center gap-2 transition-colors">
                <Download class="w-4 h-4" /> Download VB-Cable
              </button>
            </div>
          </div>

          <!-- AUDIO SECTION -->
          <div v-if="currentSection === 'audio'" class="space-y-6">
            
            <!-- Output Device -->
            <div class="bg-surface-bright rounded-2xl p-4 space-y-4 shadow-sm">
              <div>
                <h4 class="font-bold text-on-surface">Output Device</h4>
                <p class="text-xs text-on-surface-variant">Where should the microphone audio be played?</p>
              </div>
              <div class="relative">
                <select v-model="settings.audioDevice" class="w-full bg-surface-container rounded-xl px-4 py-3 text-sm border-none outline-none appearance-none font-medium cursor-pointer">
                  <option value="">System Default</option>
                  <option v-for="dev in audioDevices" :key="dev" :value="dev">{{ dev }}</option>
                </select>
                <ChevronDown class="w-4 h-4 absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none text-on-surface-variant" />
              </div>
              <p v-if="settings.audioDevice.includes('CABLE Input')" class="text-xs text-green-400 font-medium">
                VB-CABLE routing is active. Audio will be forwarded to "CABLE Output".
              </p>
            </div>

            <!-- Spectrum Analyzer / Real-time Monitoring -->
            <div class="bg-surface-bright rounded-2xl p-4 space-y-3 shadow-sm">
              <div class="flex justify-between items-center mb-2">
                <h4 class="font-bold text-on-surface text-sm">Real-Time Spectrum</h4>
                <div class="flex gap-4">
                  <div class="flex items-center gap-2">
                    <div class="w-3 h-3 rounded-sm bg-surface-variant/50"></div>
                    <span class="text-[10px] text-on-surface-variant">原始 (Raw)</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-3 h-3 rounded-sm bg-primary"></div>
                    <span class="text-[10px] text-on-surface-variant">处理后 (Processed)</span>
                  </div>
                </div>
              </div>
              <div class="w-full h-32 bg-surface-container/30 rounded-xl overflow-hidden relative">
                <canvas ref="spectrumCanvas" class="w-full h-full"></canvas>
              </div>
            </div>

            <!-- Amplifier (Gain) -->
            <div class="bg-surface-bright rounded-2xl p-4 shadow-sm flex items-center gap-4">
              <span class="text-sm font-medium text-on-surface whitespace-nowrap">Gain</span>
              <input type="range" min="-50" max="50" v-model.number="settings.gain" class="w-full accent-primary">
              <span class="text-xs w-12 text-right">{{ settings.gain > 0 ? '+' : '' }}{{ settings.gain }} dB</span>
            </div>

            <!-- Noise Suppression -->
            <div class="bg-surface-bright rounded-2xl p-4 shadow-sm space-y-4">
              <div class="flex justify-between items-center cursor-pointer" @click="settings.nsEnabled = !settings.nsEnabled">
                <span class="font-medium text-on-surface">Noise Suppression</span>
                <div class="w-10 h-5 rounded-full relative transition-colors" :class="settings.nsEnabled ? 'bg-primary' : 'bg-surface-variant'">
                  <div class="w-5 h-5 bg-white rounded-full absolute shadow-sm transition-all" :class="settings.nsEnabled ? 'right-0' : 'left-0'"></div>
                </div>
              </div>
              <div v-if="settings.nsEnabled" class="space-y-4 pt-2 border-t border-surface-variant/20">
                <div class="flex gap-2 mt-4">
                  <button v-for="type in [{id: 'Ulunas', label: 'Ulunas (ONNX)'}, {id: 'RNNoise', label: 'RNNoise'}, {id: 'Speexdsp', label: 'Speexdsp'}, {id: 'Lightweight', label: 'Lightweight'}]" :key="type.id"
                          @click="settings.nsType = type.id"
                          class="px-3 py-1 rounded-full text-xs font-medium transition-colors"
                          :class="settings.nsType === type.id ? 'bg-primary text-on-primary' : 'bg-surface-container text-on-surface'">
                    {{ type.label }}
                  </button>
                </div>
                <div class="flex items-center gap-4">
                  <span class="text-xs text-on-surface-variant whitespace-nowrap">Intensity</span>
                  <input type="range" min="0" max="100" v-model.number="settings.nsIntensity" class="w-full accent-primary">
                  <span class="text-xs w-8 text-right">{{ settings.nsIntensity }}%</span>
                </div>
              </div>
            </div>

            <!-- Dereverb -->
            <div class="bg-surface-bright rounded-2xl p-4 shadow-sm space-y-4">
              <div class="flex justify-between items-center cursor-pointer" @click="settings.dereverbEnabled = !settings.dereverbEnabled">
                <span class="font-medium text-on-surface">Dereverb</span>
                <div class="w-10 h-5 rounded-full relative transition-colors" :class="settings.dereverbEnabled ? 'bg-primary' : 'bg-surface-variant'">
                  <div class="w-5 h-5 bg-white rounded-full absolute shadow-sm transition-all" :class="settings.dereverbEnabled ? 'right-0' : 'left-0'"></div>
                </div>
              </div>
              <div v-if="settings.dereverbEnabled" class="flex items-center gap-4 pt-4 border-t border-surface-variant/20">
                <span class="text-xs text-on-surface-variant whitespace-nowrap">Level</span>
                <input type="range" min="0" max="100" v-model.number="settings.dereverbLevel" class="w-full accent-primary">
                <span class="text-xs w-8 text-right">{{ settings.dereverbLevel }}%</span>
              </div>
            </div>

            <!-- Auto Gain Control -->
            <div class="bg-surface-bright rounded-2xl p-4 shadow-sm space-y-4">
              <div class="flex justify-between items-center cursor-pointer" @click="settings.agcEnabled = !settings.agcEnabled">
                <span class="font-medium text-on-surface">Auto Gain Control</span>
                <div class="w-10 h-5 rounded-full relative transition-colors" :class="settings.agcEnabled ? 'bg-primary' : 'bg-surface-variant'">
                  <div class="w-5 h-5 bg-white rounded-full absolute shadow-sm transition-all" :class="settings.agcEnabled ? 'right-0' : 'left-0'"></div>
                </div>
              </div>
              <div v-if="settings.agcEnabled" class="space-y-4 pt-4 border-t border-surface-variant/20">
                <div class="flex items-center gap-4">
                  <span class="text-xs text-on-surface-variant w-20">Target</span>
                  <input type="range" min="0" max="32767" v-model.number="settings.agcTarget" class="w-full accent-primary">
                  <span class="text-xs w-10 text-right">{{ settings.agcTarget }}</span>
                </div>
                <div class="flex items-center gap-4">
                  <span class="text-xs text-on-surface-variant w-20">Attack</span>
                  <input type="range" min="1" max="100" v-model.number="settings.agcAttack" class="w-full accent-primary">
                  <span class="text-xs w-10 text-right">{{ (settings.agcAttack / 1000).toFixed(3) }}</span>
                </div>
                <div class="flex items-center gap-4">
                  <span class="text-xs text-on-surface-variant w-20">Decay</span>
                  <input type="range" min="1" max="100" v-model.number="settings.agcDecay" class="w-full accent-primary">
                  <span class="text-xs w-10 text-right">{{ (settings.agcDecay / 10000).toFixed(4) }}</span>
                </div>
              </div>
            </div>

            <!-- Voice Activity Detection -->
            <div class="bg-surface-bright rounded-2xl p-4 shadow-sm space-y-4">
              <div class="flex justify-between items-center cursor-pointer" @click="settings.vadEnabled = !settings.vadEnabled">
                <span class="font-medium text-on-surface">Voice Activity Detection</span>
                <div class="w-10 h-5 rounded-full relative transition-colors" :class="settings.vadEnabled ? 'bg-primary' : 'bg-surface-variant'">
                  <div class="w-5 h-5 bg-white rounded-full absolute shadow-sm transition-all" :class="settings.vadEnabled ? 'right-0' : 'left-0'"></div>
                </div>
              </div>
              <div v-if="settings.vadEnabled" class="flex items-center gap-4 pt-4 border-t border-surface-variant/20">
                <span class="text-xs text-on-surface-variant whitespace-nowrap">Threshold</span>
                <input type="range" min="-100" max="0" v-model.number="settings.vadThreshold" class="w-full accent-primary">
                <span class="text-xs w-12 text-right">{{ settings.vadThreshold }} dB</span>
              </div>
            </div>

          </div>
          
          <!-- PLUGINS & ABOUT (TODO placeholders) -->
          <div v-if="['plugins', 'about'].includes(currentSection)" class="flex flex-col items-center justify-center py-12 text-center opacity-50">
            <Construction class="w-16 h-16 mb-4 text-on-surface-variant" />
            <h4 class="text-lg font-bold">Under Construction</h4>
            <p class="text-sm">This section is being ported from the KMP version.</p>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { 
  Settings as SettingsIcon, 
  X, 
  Mic, 
  Puzzle, 
  Info,
  ChevronDown,
  Download,
  Construction
} from 'lucide-vue-next';

const props = defineProps<{
  isOpen: boolean
}>();

const emit = defineEmits(['close', 'updateDevice']);

const sections = [
  { id: 'general', name: 'General', icon: SettingsIcon },
  { id: 'audio', name: 'Audio', icon: Mic },
  { id: 'plugins', name: 'Plugins', icon: Puzzle },
  { id: 'about', name: 'About', icon: Info },
];

const currentSection = ref('general');
const currentSectionName = computed(() => sections.find(s => s.id === currentSection.value)?.name);

// Reactive Settings State
const settings = reactive({
  audioDevice: '',
  gain: 0,
  nsEnabled: false,
  nsType: 'RNNoise',
  nsIntensity: 50,
  dereverbEnabled: false,
  dereverbLevel: 50,
  agcEnabled: false,
  agcTarget: 16000,
  agcAttack: 50,
  agcDecay: 50,
  vadEnabled: false,
  vadThreshold: -40
});

const audioDevices = ref<string[]>([]);
const hasVBCable = computed(() => audioDevices.value.some(d => d.toLowerCase().includes('cable')));
const audioLevel = ref(0);
let unlistenLevel: UnlistenFn | null = null;
let unlistenSpectrum: UnlistenFn | null = null;

const spectrumCanvas = ref<HTMLCanvasElement | null>(null);
let animationFrameId: number;

// Real spectrum data from backend
const rawSpectrum = ref<number[]>(new Array(64).fill(0));
const processedSpectrum = ref<number[]>(new Array(64).fill(0));

interface SpectrumPayload {
  raw: number[];
  processed: number[];
}

const drawSpectrum = () => {
  if (!spectrumCanvas.value) {
    animationFrameId = requestAnimationFrame(drawSpectrum);
    return;
  }
  
  const canvas = spectrumCanvas.value;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  if (canvas.width !== rect.width * dpr || canvas.height !== rect.height * dpr) {
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
  }

  const width = rect.width;
  const height = rect.height;
  
  ctx.clearRect(0, 0, width, height);

  const raw = rawSpectrum.value;
  const proc = processedSpectrum.value;
  const barCount = raw.length;
  const gap = 2;
  const barWidth = width / barCount;
  const effectiveBarWidth = barWidth - gap;

  for (let i = 0; i < barCount; i++) {
    const rawH = (raw[i] || 0) * height;
    const procH = (proc[i] || 0) * height;

    if (rawH > 0.5) {
      ctx.fillStyle = 'rgba(150, 150, 150, 0.25)';
      ctx.beginPath();
      ctx.roundRect(i * barWidth + gap/2, height - rawH, effectiveBarWidth, rawH, 2);
      ctx.fill();
    }

    if (procH > 0.5) {
      const gradient = ctx.createLinearGradient(0, height - procH, 0, height);
      gradient.addColorStop(0, 'rgba(74, 103, 45, 0.7)');
      gradient.addColorStop(1, 'rgba(74, 103, 45, 1)');
      
      ctx.fillStyle = gradient;
      ctx.beginPath();
      ctx.roundRect(i * barWidth + gap/2, height - procH, effectiveBarWidth, procH, 2);
      ctx.fill();
    }
  }

  animationFrameId = requestAnimationFrame(drawSpectrum);
};

onMounted(() => {
  animationFrameId = requestAnimationFrame(drawSpectrum);
});

onUnmounted(() => {
  cancelAnimationFrame(animationFrameId);
});

const fetchDevices = async () => {
  try {
    audioDevices.value = await invoke<string[]>('get_audio_devices');
  } catch (e) {
    console.error("Failed to fetch audio devices", e);
  }
};

const loadSettings = () => {
  const saved = localStorage.getItem('micyou_audio_settings');
  if (saved) {
    try {
      Object.assign(settings, JSON.parse(saved));
    } catch (e) {
      console.error("Failed to parse settings", e);
    }
  }
  
  // Legacy support
  const savedDevice = localStorage.getItem('micyou_output_device');
  if (savedDevice && !settings.audioDevice) {
    settings.audioDevice = savedDevice;
  }
  
  if (settings.audioDevice) {
    emit('updateDevice', settings.audioDevice);
  }
};

const syncSettingsToBackend = async () => {
  try {
    await invoke('update_audio_settings', {
      settings: {
        gain: settings.gain,
        nsEnabled: settings.nsEnabled,
        nsIntensity: settings.nsIntensity,
        dereverbEnabled: settings.dereverbEnabled,
        dereverbLevel: settings.dereverbLevel,
        agcEnabled: settings.agcEnabled,
        agcTarget: settings.agcTarget,
        agcAttack: settings.agcAttack,
        agcDecay: settings.agcDecay,
        vadEnabled: settings.vadEnabled,
        vadThreshold: settings.vadThreshold,
      }
    });
  } catch (e) {
    console.error('Failed to sync DSP settings to backend:', e);
  }
};

const saveSettings = () => {
  localStorage.setItem('micyou_audio_settings', JSON.stringify(settings));
  localStorage.setItem('micyou_output_device', settings.audioDevice);
  emit('updateDevice', settings.audioDevice);
  syncSettingsToBackend();
};

watch(settings, () => {
  saveSettings();
}, { deep: true });

watch(() => props.isOpen, async (newVal) => {
  if (newVal) {
    await fetchDevices();
    loadSettings();
    // Sync existing settings to backend on open
    await syncSettingsToBackend();
    unlistenLevel = await listen<number>('audio-level', (event) => {
      audioLevel.value = event.payload;
    });
    unlistenSpectrum = await listen<SpectrumPayload>('audio-spectrum', (event) => {
      rawSpectrum.value = event.payload.raw;
      processedSpectrum.value = event.payload.processed;
    });
  } else {
    if (unlistenLevel) {
      unlistenLevel();
      unlistenLevel = null;
    }
    if (unlistenSpectrum) {
      unlistenSpectrum();
      unlistenSpectrum = null;
    }
    audioLevel.value = 0;
    rawSpectrum.value = new Array(64).fill(0);
    processedSpectrum.value = new Array(64).fill(0);
  }
});
</script>
