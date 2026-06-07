<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-8 bg-black/40 backdrop-blur-sm">
    <div class="haze-surface w-full max-w-5xl h-full max-h-[80vh] rounded-3xl flex overflow-hidden shadow-2xl relative border border-white/10">
      
      <!-- Close Button -->
      <button @click="$emit('close')" class="absolute top-4 right-4 z-10 w-10 h-10 rounded-full bg-surface-variant/40 hover:bg-surface-variant/80 flex items-center justify-center transition-colors">
        <X class="w-5 h-5 text-on-surface" />
      </button>

      <!-- Left Sidebar -->
      <div class="w-64 bg-surface-container-low/50 border-r border-white/5 flex flex-col p-4 space-y-2 overflow-y-auto">
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
      <div class="flex-1 bg-surface-container-lowest/30 p-8 overflow-y-auto">
        <div class="max-w-2xl mx-auto space-y-8">
          <h3 class="text-3xl font-bold text-primary mb-6">{{ currentSectionName }}</h3>

          <!-- GENERAL SECTION -->
          <div v-if="currentSection === 'general'" class="space-y-6">
            <!-- Language (TODO) -->
            <div class="bg-surface-bright/50 rounded-2xl p-4 flex items-center justify-between">
              <div>
                <h4 class="font-bold text-on-surface">Language</h4>
                <p class="text-xs text-on-surface-variant">Choose application language</p>
              </div>
              <select class="bg-surface-container rounded-lg px-3 py-2 text-sm border-none outline-none">
                <option>English</option>
                <option>简体中文</option>
              </select>
            </div>

            <!-- Auto Start (TODO) -->
            <div class="bg-surface-bright/50 rounded-2xl p-4 flex items-center justify-between">
              <div>
                <h4 class="font-bold text-on-surface">Launch on Startup</h4>
                <p class="text-xs text-on-surface-variant">Start MicYou automatically when you log in</p>
              </div>
              <!-- TODO: Toggle switch -->
              <div class="w-10 h-5 bg-surface-variant rounded-full relative opacity-50 cursor-not-allowed">
                <div class="w-5 h-5 bg-on-surface-variant rounded-full absolute left-0 shadow-sm"></div>
              </div>
            </div>

            <!-- VB-CABLE Management -->
            <div class="bg-surface-bright/50 rounded-2xl p-4 space-y-4">
              <div class="flex items-center justify-between">
                <h4 class="font-bold text-on-surface text-lg">VB-Cable Forwarding</h4>
                <span class="text-xs font-medium px-2 py-1 rounded-md" :class="hasVBCable ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'">
                  {{ hasVBCable ? 'Installed' : 'Not Detected' }}
                </span>
              </div>
              <p class="text-xs text-on-surface-variant">
                To route audio to other applications (like OBS or Discord), you need to install VB-Audio Virtual Cable. 
                Select "CABLE Input" in the Audio settings, and "CABLE Output" in your target application.
              </p>
              <button v-if="!hasVBCable" class="w-full py-2 bg-surface-variant hover:bg-surface-variant/80 rounded-xl text-sm font-bold flex items-center justify-center gap-2 transition-colors">
                <Download class="w-4 h-4" /> Download VB-Cable
              </button>
            </div>
          </div>

          <!-- AUDIO SECTION -->
          <div v-if="currentSection === 'audio'" class="space-y-6">
            <div class="bg-surface-bright/50 rounded-2xl p-4 space-y-4">
              <div>
                <h4 class="font-bold text-on-surface">Output Device</h4>
                <p class="text-xs text-on-surface-variant">Where should the microphone audio be played?</p>
              </div>
              <div class="relative">
                <select v-model="selectedAudioDevice" @change="saveAudioDevice" class="w-full bg-surface-container rounded-xl px-4 py-3 text-sm border-none outline-none appearance-none font-medium cursor-pointer">
                  <option value="">System Default</option>
                  <option v-for="dev in audioDevices" :key="dev" :value="dev">{{ dev }}</option>
                </select>
                <ChevronDown class="w-4 h-4 absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none text-on-surface-variant" />
              </div>
              <p v-if="selectedAudioDevice.includes('CABLE Input')" class="text-xs text-green-400 font-medium">
                VB-CABLE routing is active. Audio will be forwarded to "CABLE Output".
              </p>
            </div>

            <!-- AEC (TODO) -->
            <div class="bg-surface-bright/50 rounded-2xl p-4 flex items-center justify-between opacity-60">
              <div>
                <h4 class="font-bold text-on-surface">Noise Suppression (NS)</h4>
                <p class="text-xs text-on-surface-variant">Reduce background noise from microphone (Coming soon)</p>
              </div>
              <div class="w-10 h-5 bg-primary/30 rounded-full relative">
                <div class="w-5 h-5 bg-primary rounded-full absolute right-0 shadow-sm"></div>
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
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
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

const audioDevices = ref<string[]>([]);
const selectedAudioDevice = ref<string>('');
const hasVBCable = computed(() => audioDevices.value.some(d => d.toLowerCase().includes('cable')));

const fetchDevices = async () => {
  try {
    audioDevices.value = await invoke<string[]>('get_audio_devices');
    // Initialize selected device from localStorage if exists
    const savedDevice = localStorage.getItem('micyou_output_device');
    if (savedDevice && audioDevices.value.includes(savedDevice)) {
      selectedAudioDevice.value = savedDevice;
      emit('updateDevice', savedDevice);
    }
  } catch (e) {
    console.error("Failed to fetch audio devices", e);
  }
};

const saveAudioDevice = () => {
  localStorage.setItem('micyou_output_device', selectedAudioDevice.value);
  emit('updateDevice', selectedAudioDevice.value);
};

watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    fetchDevices();
  }
});
</script>
