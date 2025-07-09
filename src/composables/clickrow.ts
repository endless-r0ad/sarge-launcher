import { ref, type Ref } from 'vue'
import { type Nullable } from '@/utils/util'
import { type Quake3Server } from '@/models/server'
import { type Demo } from '@/models/demo'
import { type Level } from '@/models/level'

export function useClickRow(
  selected: Ref<Nullable<Level> | Nullable<Demo> | Nullable<Quake3Server>>,
  lastSelected: Ref<Nullable<Level> | Nullable<Demo> | Nullable<Quake3Server>>,
  displayDetails: Ref<boolean> | null = null
) {
  const dblClickTimer = ref<Nullable<number>>(null)
  const numClicks = ref(0)
  const dblClickHappenedOnSameObject = ref(false)

  function handleClick(clicked: Level | Demo | Quake3Server, target: HTMLTextAreaElement | null = null) {
    numClicks.value++
    lastSelected.value = selected.value
    selected.value = clicked
    if (numClicks.value == 1) {
      dblClickTimer.value = window.setTimeout(() => {
        numClicks.value = 0
        dblClickHappenedOnSameObject.value = false
        if (selected.value == lastSelected.value && target?.id != 'expandDetails') {
          selected.value = null
          if (displayDetails) {
            displayDetails.value = false
          }
        }
      }, 250)
    }
    if (numClicks.value == 2) {
      if (selected.value == lastSelected.value) {
        dblClickHappenedOnSameObject.value = true
      }
    }
  }

  function rightClickToSelect(clicked: Level | Demo | Quake3Server) {
    lastSelected.value = selected.value
    selected.value = clicked
  }

  function resetDblClickTimeout() {
    if (dblClickTimer.value) {
      clearTimeout(dblClickTimer.value)
      numClicks.value = 0
      dblClickHappenedOnSameObject.value = false
    }
  }

  return {
    numClicks,
    handleClick,
    dblClickHappenedOnSameObject,
    resetDblClickTimeout,
    rightClickToSelect,
  }
}
