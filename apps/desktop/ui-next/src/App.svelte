<script lang="ts">
  import { onMount } from 'svelte';
  import { greet } from '@/types/commands';
  import SettingsLayout from '@/components/SettingsLayout.svelte';

  let greeting = $state('');
  let name = $state('Tauri + Svelte 5');
  let showSettings = $state(false);

  async function handleGreet() {
    try {
      greeting = await greet(name);
    } catch (e) {
      greeting = `Error: ${e}`;
    }
  }

  onMount(() => {
    console.log('POC: Vite + Svelte 5 + Tauri initialized');
  });
</script>

{#if showSettings}
  <SettingsLayout />
{:else}
  <main class="container mx-auto px-4 py-8">
    <div class="max-w-2xl mx-auto">
      <div class="text-center mb-8">
        <h1 class="text-4xl font-bold mb-4">UI Rewrite POC</h1>
        <p class="text-gray-600 mb-8">Vite + Svelte 5 + TailwindCSS v4 + Tauri</p>
        
        <div class="space-y-4">
          <input 
            type="text" 
            bind:value={name}
            placeholder="Enter name..."
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          
          <button 
            onclick={handleGreet}
            class="w-full px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Greet
          </button>
          
          {#if greeting}
            <p class="mt-4 p-4 bg-green-50 border border-green-200 rounded-lg text-green-800">
              {greeting}
            </p>
          {/if}
        </div>
      </div>

      <div class="mt-8 border-t pt-8">
        <button
          onclick={() => showSettings = true}
          class="w-full px-6 py-3 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
        >
          Open Settings (Phase 3)
        </button>
      </div>
    </div>
  </main>
{/if}

{#if showSettings}
  <button
    onclick={() => showSettings = false}
    class="fixed top-4 right-4 px-4 py-2 bg-gray-800 text-white rounded-lg hover:bg-gray-900 transition-colors z-50"
  >
    ← Back to Demo
  </button>
{/if}
