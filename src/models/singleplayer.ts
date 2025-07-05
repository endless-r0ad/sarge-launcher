import { Q3_BOT_NAMES } from "@/utils/util"

export interface Bot {
  name: (typeof Q3_BOT_NAMES)[number]
  difficulty: number
  team: 'Free' | 'Red' | 'Blue'
}
