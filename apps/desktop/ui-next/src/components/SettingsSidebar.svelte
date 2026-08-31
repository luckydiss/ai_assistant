<script lang="ts">
  import { SETTINGS_NAV, type SettingsSection } from '@/types/navigation';

  interface Props {
    active: SettingsSection;
    onNavigate: (section: SettingsSection) => void;
  }

  let { active, onNavigate }: Props = $props();

  // Group items by group name
  const grouped = SETTINGS_NAV.reduce((acc, item) => {
    if (!acc[item.group]) acc[item.group] = [];
    acc[item.group].push(item);
    return acc;
  }, {} as Record<string, typeof SETTINGS_NAV>);
</script>

<nav class="settings-sidebar w-64 bg-gray-50 border-r border-gray-200 p-4">
  <h2 class="text-lg font-semibold mb-4">Настройки</h2>
  
  <div class="space-y-6">
    {#each Object.entries(grouped) as [groupName, items]}
      <div>
        <h3 class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
          {groupName}
        </h3>
        <div class="space-y-1">
          {#each items as item}
            <button
              onclick={() => onNavigate(item.id)}
              class="w-full text-left px-3 py-2 rounded-lg text-sm transition-colors {active === item.id
                ? 'bg-blue-100 text-blue-700 font-medium'
                : 'text-gray-700 hover:bg-gray-100'}"
            >
              {item.label}
            </button>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</nav>
