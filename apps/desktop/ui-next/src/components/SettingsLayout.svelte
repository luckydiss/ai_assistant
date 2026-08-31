<script lang="ts">
  import { type SettingsSection } from '@/types/navigation';
  import SettingsSidebar from './SettingsSidebar.svelte';
  import AudioSettings from './AudioSettings.svelte';
  import HotkeysSettings from './HotkeysSettings.svelte';
  import WindowSettings from './WindowSettings.svelte';
  import ChatSettings from './ChatSettings.svelte';
  import ProtectionSettings from './ProtectionSettings.svelte';
  import TtsControls from '@/lib/TtsControls.svelte';

  let activeSection = $state<SettingsSection>('audio');

  function handleNavigate(section: SettingsSection) {
    activeSection = section;
  }
</script>

<div class="settings-layout flex h-screen bg-white">
  <SettingsSidebar active={activeSection} onNavigate={handleNavigate} />
  
  <main class="flex-1 overflow-y-auto">
    <div class="max-w-3xl mx-auto py-8">
      {#if activeSection === 'audio'}
        <AudioSettings />
      {:else if activeSection === 'tts'}
        <TtsControls />
      {:else if activeSection === 'hotkeys'}
        <HotkeysSettings />
      {:else if activeSection === 'window'}
        <WindowSettings />
      {:else if activeSection === 'chat'}
        <ChatSettings />
      {:else if activeSection === 'protection'}
        <ProtectionSettings />
      {/if}
    </div>
  </main>
</div>
