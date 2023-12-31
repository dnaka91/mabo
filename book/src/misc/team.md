---
layout: page
---
<script setup>
import {
  VPTeamPage,
  VPTeamPageTitle,
  VPTeamMembers
} from "vitepress/theme"

const members = [
  {
    avatar: "https://avatars.githubusercontent.com/u/36804488",
    name: "Dominik Nakamura",
    title: "Creator",
    links: [
      { icon: "github", link: "https://github.com/dnaka91" },
      { icon: "discord", link: "https://discord.gg/phxGsW8dWd" },
      { icon: "linkedin", link: "https://www.linkedin.com/in/dominik-nakamura" }
    ],
    sponsor: "https://github.com/sponsors/dnaka91"
  }
]
</script>

<VPTeamPage>
  <VPTeamPageTitle>
    <template #title>
      Our Team
    </template>
    <template #lead>
      Currently the team consists of only one person.
    </template>
  </VPTeamPageTitle>
  <VPTeamMembers
    :members="members"
  />
</VPTeamPage>
