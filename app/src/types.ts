export type ResearchResult = {
  path: string;
  title: string | null;
  priority: number;
};

export type AppInfo = {
  version: string;
};

export type MenuEntry = { action: string; icon: string; keymap: string };
