// Navigation types for settings sections

export type SettingsSection = 
  | 'audio'
  | 'tts'
  | 'hotkeys'
  | 'chat'
  | 'window'
  | 'protection';

export interface NavItem {
  id: SettingsSection;
  label: string;
  group: string;
  icon?: string;
}

export const SETTINGS_NAV: NavItem[] = [
  { id: 'audio', label: 'Запись звука', group: 'Аудио' },
  { id: 'tts', label: 'Озвучка ответов', group: 'TTS' },
  { id: 'hotkeys', label: 'Горячие клавиши', group: 'Управление' },
  { id: 'chat', label: 'Чат', group: 'Интерфейс' },
  { id: 'window', label: 'Окно', group: 'Интерфейс' },
  { id: 'protection', label: 'Защита', group: 'Интерфейс' },
];
