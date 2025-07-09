import { type ComputedRef, useTemplateRef, ref, computed, onActivated, onDeactivated } from 'vue'
import { type Nullable } from '@/utils/util'

export interface VirtualScroll {
  rowHeight: number
  overscan: number
}

export function useVirtualScroll(
  scrollableDiv: string,
  totalItems: ComputedRef<number>,
  pinnedToTop: Nullable<ComputedRef<number>> = null
) {
  const scroller = ref<VirtualScroll>({ rowHeight: 24, overscan: 10 })
  const setScroller = (newScroller: VirtualScroll) => {
    scroller.value = newScroller
  }

  const scrollingTable = useTemplateRef<HTMLDivElement>(scrollableDiv)

  const translateY = ref(0)
  const viewportHeight = ref(0)

  const virtualHeight = computed(() => {
    return scroller.value.rowHeight * totalItems.value + scroller.value.rowHeight
  })

  const virtualStartIndex = computed(() => {
    return Math.max(0, translateY.value / scroller.value.rowHeight - (scroller.value.overscan + (pinnedToTop?.value ?? 0)))
  })

  const virtualEndIndex = computed(() => {
    return Math.min(
      totalItems.value,
      Math.floor((translateY.value + viewportHeight.value) / scroller.value.rowHeight + scroller.value.overscan)
    )
  })

  const marginTop = computed(() => {
    return translateY.value > scroller.value.overscan * scroller.value.rowHeight + (pinnedToTop?.value ?? 0) * scroller.value.rowHeight
      ? -scroller.value.rowHeight * (scroller.value.overscan + (pinnedToTop?.value ?? 0))
      : -translateY.value
  })

  function handleScroll() {
    if (scrollingTable.value) {
      translateY.value = Math.min(
        virtualHeight.value,
        scroller.value.rowHeight * Math.floor(scrollingTable.value.scrollTop / scroller.value.rowHeight)
      )
    }
  }

  function handleResize() {
    if (scrollingTable.value) {
      viewportHeight.value = scrollingTable.value.clientHeight
    }
  }

  onActivated(async () => {
    scrollingTable.value?.addEventListener('scroll', handleScroll)
    window.addEventListener('resize', handleResize)
    handleResize()
    handleScroll()
  })

  onDeactivated(async () => {
    scrollingTable.value?.removeEventListener('scroll', handleScroll)
    window.removeEventListener('resize', handleResize)
  })

  return {
    scroller,
    setScroller,
    translateY,
    virtualHeight,
    marginTop,
    virtualStartIndex,
    virtualEndIndex,
    handleScroll,
  }
}
