# RPG-Combat-TUI

# Usage
Two files are required:
- `players.yml`: Here you define your player stats. Should be one file per adventure/group.
- `combat_name.yml`: Here you define the monsters for a specific combat. Should be one file for each combat.
## Players file
**Example:**
```yml
players:
  - name: Merry
    initiative_modifier: 2
    ac: 16
    max_hp: 40
    current_hp: 26
  - name: Pipping
    initiative_modifier: 2
    ac: 16
    max_hp: 40
    current_hp: 104
  - name: Samwise
    initiative_modifier: -2
    ac: 12
    max_hp: 70
```
## Combat file
For each monster:
- specify the `count`
- specify the `stats`
**Example:**
```yml
monsters:
  - count: 1
    stats:
      name: BBEG
      ac: 12
      max_hp: 110
      initiative_modifier: 2
  - count: 5
    stats:
      name: Nothic
      ac: 15
      max_hp: 45
      initiative_modifier: 3
```

# TODOs
- [ ] save state in new section
- [ ] different styling for players and monsters

