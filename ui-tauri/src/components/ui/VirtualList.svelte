<script lang="ts" generics="T">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import type { ComponentType, SvelteComponent } from 'svelte';
  
  // Generic type T is defined in the script tag
  export let items: T[] = [];
  export let itemHeight: number = 40;
  export let containerHeight: number = 400;
  export let overscan: number = 5;
  export let renderItem: (item: T, index: number) => string = () => '';
  export let itemComponent: ComponentType<SvelteComponent<{ item: T; index: number }>> | null = null;
  export let selected: number = -1;
  
  interface ScrollEventDetail {
    scrollTop: number;
    visibleStart: number;
    visibleEnd: number;
  }
  
  interface SelectEventDetail {
    index: number;
    item: T;
  }
  
  interface ItemVisibleEventDetail {
    index: number;
    item: T;
  }
  
  const dispatch = createEventDispatcher<{
    scroll: ScrollEventDetail;
    select: SelectEventDetail;
    itemVisible: ItemVisibleEventDetail;
  }>();
  
  let scrollTop = 0;
  let container: HTMLElement;
  let viewport: HTMLElement;
  let totalHeight = 0;
  let visibleStart = 0;
  let visibleEnd = 0;
  let offsetY = 0;
  
  // Reactive calculations
  $: totalHeight = items.length * itemHeight;
  
  $: visibleStart = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
  $: visibleEnd = Math.min(
    items.length - 1,
    Math.ceil((scrollTop + containerHeight) / itemHeight) + overscan
  );
  
  $: offsetY = visibleStart * itemHeight;
  $: visibleItems = items.slice(visibleStart, visibleEnd + 1);
  
  // Intersection Observer for dynamic loading
  let io: IntersectionObserver | null = null;
  
  onMount(() => {
    // Setup intersection observer for lazy loading
    io = new IntersectionObserver(
      (entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            const indexAttr = entry.target.getAttribute('data-index');
            const index = parseInt(indexAttr ?? '0', 10);
            if (index >= 0 && index < items.length) {
              dispatch('itemVisible', { index, item: items[index] });
            }
          }
        });
      },
      { root: viewport, threshold: 0.1 }
    );
    
    // Add resize observer for container height changes
    const resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        containerHeight = entry.contentRect.height;
      }
    });
    
    if (container) {
      resizeObserver.observe(container);
    }
    
    return () => {
      resizeObserver.disconnect();
      io?.disconnect();
    };
  });
  
  onDestroy(() => {
    io?.disconnect();
  });
  
  function handleScroll(e: Event): void {
    const target = e.target as HTMLElement;
    scrollTop = target.scrollTop;
    dispatch('scroll', { scrollTop, visibleStart, visibleEnd });
  }
  
  function handleKeydown(e: KeyboardEvent): void {
    let newIndex = -1;
    const pageSize = Math.floor(containerHeight / itemHeight);
    
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        newIndex = Math.min(visibleStart + 1, items.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        newIndex = Math.max(visibleStart - 1, 0);
        break;
      case 'PageDown':
        e.preventDefault();
        newIndex = Math.min(visibleStart + pageSize, items.length - 1);
        break;
      case 'PageUp':
        e.preventDefault();
        newIndex = Math.max(visibleStart - pageSize, 0);
        break;
      case 'Home':
        e.preventDefault();
        newIndex = 0;
        break;
      case 'End':
        e.preventDefault();
        newIndex = items.length - 1;
        break;
    }
    
    if (newIndex >= 0 && newIndex !== visibleStart) {
      visibleStart = newIndex;
      scrollToIndex(newIndex);
      dispatch('select', { index: newIndex, item: items[newIndex] });
    }
  }
  
  function scrollToIndex(index: number): void {
    if (viewport) {
      const targetScrollTop = index * itemHeight;
      viewport.scrollTo({
        top: targetScrollTop,
        behavior: 'smooth'
      });
    }
  }
  
  // Export method for programmatic scrolling
  export function scrollToItem(index: number): void {
    scrollToIndex(index);
  }
  
  // Export method to get visible range
  export function getVisibleRange(): { start: number; end: number } {
    return { start: visibleStart, end: visibleEnd };
  }
</script>

<div 
  bind:this={container}
  class="virtual-list-container"
  style="height: {containerHeight}px;"
>
  <div 
    bind:this={viewport}
    class="virtual-list-viewport"
    on:scroll={handleScroll}
    on:keydown={handleKeydown}
    tabindex="0"
    role="listbox"
    aria-label="Virtual list"
  >
    <div 
      class="virtual-list-spacer"
      style="height: {totalHeight}px; position: relative;"
    >
      <div 
        class="virtual-list-content"
        style="transform: translateY({offsetY}px);"
      >
        {#each visibleItems as item, i (visibleStart + i)}
          <div
            class="virtual-list-item"
            style="height: {itemHeight}px;"
            data-index={visibleStart + i}
            role="option"
            aria-selected={visibleStart + i === selected}
          >
            {#if itemComponent}
              <svelte:component this={itemComponent} {item} index={visibleStart + i} />
            {:else}
              {@html renderItem(item, visibleStart + i)}
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .virtual-list-container {
    position: relative;
    overflow: hidden;
    border: 1px solid #2a3750;
    border-radius: 8px;
    background: #0c1120;
  }
  
  .virtual-list-viewport {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: #2a3750 #0c1120;
  }
  
  .virtual-list-viewport::-webkit-scrollbar {
    width: 8px;
  }
  
  .virtual-list-viewport::-webkit-scrollbar-track {
    background: #0c1120;
  }
  
  .virtual-list-viewport::-webkit-scrollbar-thumb {
    background: #2a3750;
    border-radius: 4px;
  }
  
  .virtual-list-viewport::-webkit-scrollbar-thumb:hover {
    background: #3a4858;
  }
  
  .virtual-list-spacer {
    position: relative;
  }
  
  .virtual-list-content {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
  }
  
  .virtual-list-item {
    position: absolute;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    padding: 0 12px;
    border-bottom: 1px solid #1a2332;
    transition: background-color 0.15s ease;
  }
  
  .virtual-list-item:hover {
    background: #1a2332;
  }
  
  .virtual-list-item:focus {
    outline: 2px solid #3b82f6;
    outline-offset: -2px;
  }
  
  .virtual-list-item[aria-selected="true"] {
    background: #1e3a5f;
  }
</style>
