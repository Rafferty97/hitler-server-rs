# Secret Hitler XL - Digital Implementation Test Checklist

## Pre-Game Setup Validation

### Player Count & Configuration
- [ ] System accepts 6-20 players
- [ ] System rejects games with less than 6 or more than 20 players
- [ ] Player ratios are correctly enforced according to the table:
  - [ ] 6 players: 3L, 1F+H, 1C
  - [ ] 7 players: 4L, 1F+H, 1C
  - [ ] 8 players: 4L, 2F+H, 1C
  - [ ] 9 players: 4L, 2F+H, 2C
  - [ ] 10 players: 5L, 2F+H, 2C
  - [ ] 11 players: 5L, 3F+H, 2C
  - [ ] 12 players: 6L, 3F+H, 2C
  - [ ] 13 players: 6L, 3F+H, 3C
  - [ ] 14 players: 7L, 3F+H, 3C
  - [ ] 15 players: 7L, 4F+H, 3C
  - [ ] 16 players: 7L, 4F+H, 4C

### Policy Deck Construction
- [ ] Standard setup (not 8 players): 8 Communist, 5 Liberal, 10 Fascist policies
- [ ] 8-player setup: 8 Communist, 6 Liberal, 9 Fascist policies
- [ ] Policy deck is properly shuffled at game start
- [ ] Anti-policies are correctly included when enabled
- [ ] Emergency power cards are included correctly (1 per player above 10, 2 per player above 13 if using Communists)
- [ ] Maximum 6 emergency powers enforced (3 Article 48, 3 Enabling Act)

### Role Assignment
- [ ] Hitler is correctly assigned to one Fascist
- [ ] Communists know each other at start if 11+ players
- [ ] Communists don't know each other at start if <11 players
- [ ] Special roles (Capitalist, Anarchist, Monarchist) are assigned when enabled
- [ ] Role cards are distributed face-down and secretly

## Communist Party Mechanics

### Communist Win Conditions
- [ ] Communists win when communist policy tracker is full
- [ ] Communists and Liberals both win if Hitler is assassinated by Communists
- [ ] Game correctly identifies communist victory

### Communist Powers Implementation

#### Confession Power
- [ ] Sitting president must reveal party membership to everyone
- [ ] All players can see the revealed card
- [ ] Card is returned to player after reveal

#### Bugging Power
- [ ] All players place cards face down
- [ ] Only Communists can see during "eyes closed" phase
- [ ] Communists collectively choose one player's card to view
- [ ] Card shuffling phase works correctly
- [ ] Non-communist players cannot see the revealed information

#### Five-Year Plan Power
- [ ] Exactly 2 Communist policies and 1 Liberal policy added to draw deck
- [ ] Draw deck is shuffled after addition
- [ ] Policy count tracking remains accurate

#### Congress Power
- [ ] Only works if there are "newly radicalized" Communists
- [ ] All Communists learn who the original Communists are
- [ ] Information is revealed only to Communist players

#### Radicalization Power
- [ ] Communists can swap any player's role card with a Communist one
- [ ] Hitler cannot be turned Communist (automatically swaps back)
- [ ] Capitalist cannot be turned Communist (automatically swaps back)
- [ ] Original card is placed in center of table
- [ ] Player is notified they are now Communist
- [ ] Card shuffling phase prevents detection

### Communist Policy Tracker
- [ ] 6-8 players: 5 slots (Bugging, Radicalization, 5-year plan, Congress)
- [ ] 9-10 players: 6 slots (Bugging, Radicalization, 5-year plan, Congress, Confession)
- [ ] 11+ players: 6 slots (blank, Radicalization, 5-year plan, Radicalization, Confession)
- [ ] Alternative 3-slot tracker works with modified policy deck (10F, 4L, 6C)
- [ ] Powers activate at correct tracker positions

## Special Roles

### The Capitalist
- [ ] Wins if neither Anarchists nor Communists win
- [ ] When assassinated, additional Communist policy is shuffled into deck
- [ ] Cannot be turned Communist via Radicalization
- [ ] Role is hidden from other players

### The Anarchist (Without Communists)
- [ ] Wins if Anarchist policy enacted via election tracker
- [ ] When Anarchist policy enacted, new Anarchist policy added to deck
- [ ] Election tracker moves to end when Anarchist policy enacted
- [ ] Policy deck starts with 3 Anarchist policies
- [ ] Top card of draw pile enacted automatically

### The Anarchist (With Communists)
- [ ] Replaces 2 Communist policies with 2 Anarchist policies in deck
- [ ] Anarchist policy goes on Communist tracker but no power activated
- [ ] Wins if Communists win AND 2 Anarchist policies on Communist tracker
- [ ] Also wins if Hitler is assassinated

### The Monarchist
- [ ] Wins if Fascists win AND Hitler never becomes Chancellor after 3 Fascist policies
- [ ] Loses if Hitler is assassinated
- [ ] Must protect Hitler while preventing his rise to power

## Anti-Policies

### Anti-Communist Policy
- [ ] Placed on Fascist policy tracker when enacted
- [ ] President uses their power then removes one Communist policy
- [ ] Next Communist policy enacted doesn't trigger power reuse
- [ ] Properly tracked and displayed

### Anti-Fascist Policy
- [ ] Placed on Communist policy tracker when enacted
- [ ] President/Communists use Communist power then remove one Fascist policy
- [ ] Next Fascist policy enacted doesn't trigger power reuse
- [ ] Properly tracked and displayed

### Social Democratic Policy
- [ ] Placed on Liberal policy tracker when enacted
- [ ] President removes either one Fascist or one Communist policy
- [ ] No presidential powers may be reused due to this policy
- [ ] Only enabled when Liberals are at significant disadvantage

## Emergency Powers

### Article 48 Powers (President)
- [ ] **Propaganda**: President can secretly view and discard/replace top card
- [ ] **Policy Peek**: President views top 3 cards without reordering
- [ ] **Impeachment**: Chancellor reveals party card to President's chosen player
- [ ] **Marked for Execution**: Target player executed after 3 Fascist policies
- [ ] **Execution**: President executes player immediately
- [ ] **Presidential Pardon**: Removes Mark for Execution from chosen player

### Enabling Act Powers (Chancellor)
- [ ] **Propaganda**: Chancellor can secretly view and discard/replace top card
- [ ] **Policy Peek**: Chancellor views top 3 cards without reordering
- [ ] **Impeachment**: President reveals party card to Chancellor's chosen player
- [ ] **Marked for Execution**: Target player executed after 3 Fascist policies
- [ ] **Execution**: Chancellor executes player immediately
- [ ] **Vote of No Confidence**: President's discarded card is enacted instead

### Emergency Power General Rules
- [ ] Emergency power cards removed from game after use
- [ ] Executed players cannot speak, vote, or run for office
- [ ] Secret roles not revealed unless Hitler or Capitalist executed
- [ ] Marked for Execution properly tracked and executed at correct trigger

## Game Flow & Standard Rules

### Election Process
- [ ] Presidential nomination and voting works correctly
- [ ] Chancellor nomination and voting works correctly
- [ ] Election tracker advances on failed elections
- [ ] Top policy enacted when election tracker fills

### Policy Enactment
- [ ] President draws correct number of cards
- [ ] President discards one card secretly
- [ ] Chancellor chooses from remaining cards
- [ ] Policy placed on correct tracker
- [ ] Presidential powers activate at correct thresholds

### Victory Conditions
- [ ] Liberal victory: 5 Liberal policies enacted
- [ ] Fascist victory: 6 Fascist policies enacted OR Hitler elected Chancellor after 3 Fascist policies
- [ ] Communist victory: 6 Communist policies enacted OR Hitler assassinated
- [ ] Special role victories work correctly
- [ ] Multiple party victories handled correctly (Communist + Liberal)

### Investigation & Assassination
- [ ] Investigation reveals party membership correctly
- [ ] Assassination removes player from game
- [ ] Hitler assassination triggers appropriate victory conditions
- [ ] Capitalist assassination adds Communist policy to deck

## Extended Game Features

### Liberal Disadvantage Option
- [ ] 6-slot Liberal policy tracker implemented when enabled
- [ ] Victory condition adjusted accordingly

### Game Length Calculations
- [ ] Maximum players = total policy tracker spaces - 1 (or -2 with Communists)
- [ ] System prevents games that would end too quickly
- [ ] Proper accommodation for additional players with extended rules

## User Interface & Experience

### Information Display
- [ ] Policy trackers clearly visible and accurate
- [ ] Player roles hidden appropriately
- [ ] Current president/chancellor clearly indicated
- [ ] Election tracker status visible
- [ ] Emergency power status tracked and displayed

### Player Interactions
- [ ] Secret information properly segregated by player
- [ ] "Eyes closed" phases work correctly in digital format
- [ ] Card placement and shuffling phases simulated appropriately
- [ ] Voting interface clear and functional
- [ ] Chat/communication features respect game phases

## Error Handling & Edge Cases

### Invalid Actions
- [ ] System prevents illegal moves
- [ ] Proper error messages for invalid actions
- [ ] Game state remains consistent after errors

### Disconnection Handling
- [ ] Players can reconnect and see appropriate information
- [ ] Game can continue with disconnected players
- [ ] Secret information remains secure

### End Game Scenarios
- [ ] Multiple victory conditions resolved correctly
- [ ] Game ending properly communicated to all players
- [ ] Final game state saved/displayed correctly

## Performance & Scalability

### Technical Requirements
- [ ] Game handles 20 players without performance issues
- [ ] Real-time updates work smoothly
- [ ] Secret information transmission secure
- [ ] Game state properly synchronized across all clients

---

## Test Scenarios by Player Count

### 6-Player Game Tests
- [ ] Basic 3-party setup works correctly
- [ ] Communist tracker progression (5 slots)
- [ ] All powers activate correctly

### 8-Player Game Tests  
- [ ] Modified policy deck ratios (8C, 6L, 9F)
- [ ] Standard communist tracker
- [ ] Game length appropriate for player count

### 11+ Player Game Tests
- [ ] Communists know each other at start
- [ ] Modified communist tracker (blank first slot)
- [ ] Emergency powers included correctly
- [ ] Game length extended appropriately

### 16+ Player Game Tests
- [ ] Maximum emergency powers (6 total)
- [ ] All special roles functioning
- [ ] Performance remains stable
- [ ] Game completion time reasonable

---

## Integration Testing

### Multi-Role Combinations
- [ ] Communist + Capitalist combination
- [ ] Communist + Anarchist combination
- [ ] All special roles together
- [ ] Anti-policies with special roles
- [ ] Emergency powers with special roles

### Policy Interaction Tests
- [ ] Anti-policies correctly interact with trackers
- [ ] Emergency power policies don't conflict
- [ ] Social Democratic policy interactions
- [ ] Five-Year Plan additions work correctly

---

*This checklist should be used systematically to verify that all aspects of Secret Hitler XL are properly implemented in the digital version. Each item should be tested with multiple scenarios and edge cases to ensure robust functionality.*