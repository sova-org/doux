import type { Component } from "svelte";
import navigation from "$lib/navigation.json";

const contentModules = import.meta.glob("/src/content/*.md", { eager: true });

interface ContentMetadata {
  title: string;
  slug: string;
  group: string;
  order: number;
}

interface ContentModule {
  metadata: ContentMetadata;
  default: Component;
}

export function load() {
  const categories = Object.entries(contentModules).map(([path, module]) => {
    const mod = module as ContentModule;
    return {
      path,
      ...mod.metadata,
      component: mod.default,
    };
  });

  const groupOrder = { sources: 0, synthesis: 1, effects: 2 };
  categories.sort((a, b) => {
    const groupDiff =
      (groupOrder[a.group as keyof typeof groupOrder] ?? 99) -
      (groupOrder[b.group as keyof typeof groupOrder] ?? 99);
    if (groupDiff !== 0) return groupDiff;
    return a.order - b.order;
  });

  return {
    categories,
    navigation,
  };
}
