# Secret Hitler XL Test Specification Analysis
## Ambiguities and Inconsistencies Between test.md and Rules.pdf

This document identifies discrepancies between the test.md checklist and the authoritative Secret Hitler XL Rules.pdf, highlighting areas where assumptions were made and questions that need clarification.

---

## 🔴 CRITICAL DISCREPANCIES

### 1. Player Count Limits (Lines 6-7 in test.md vs Lines 139-151 in rules.pdf)

**test.md states:** "System accepts 6-20 players" and "System rejects games with less than 6 or more than 20 players"

**rules.pdf shows:** Player ratio table only goes up to 16 players (Lines 139-151)

**AMBIGUITY:** The rules.pdf doesn't explicitly state a maximum of 20 players. The player ratio table ends at 16 players, but the document mentions accommodating "up to 20 players" in the introduction (Line 90).

**ASSUMPTION MADE:** Tests assume 6-20 player range, but implementation may need to support only 6-16 based on the actual ratio table provided.

**QUESTION:** Should the system support 17-20 players? If so, what are the role ratios for these player counts?

### 2. Role Name Inconsistency: "Monarchist" vs "Nationalist"

**test.md uses:** "Monarchist" throughout (Lines 103-106, etc.)

**rules.pdf uses:** Both "Monarchist" (Line 486) and "Nationalist" (Line 108, Line 687)

**AMBIGUITY:** The rules.pdf inconsistently refers to this role as both "Monarchist" and "Nationalist"

**ASSUMPTION MADE:** Tests use "Monarchist" as it appears more frequently in the detailed rules section

**QUESTION:** Which is the correct role name? Should both be supported as aliases?

---

## 🟡 MODERATE AMBIGUITIES

### 3. Emergency Power Distribution Rules (Lines 26-27 in test.md vs Lines 625-634 in rules.pdf)

**test.md states:** "1 per player above 10, 2 per player above 13 if using Communists"

**rules.pdf states:** "Include one Emergency power card for every player above 10. If you're including communists, include two Emergency power cards for every player above 13."

**AMBIGUITY:** The phrasing could be interpreted as:
- Option A: 1 per player above 10, PLUS 2 per player above 13 (cumulative)
- Option B: 1 per player above 10, OR 2 per player above 13 (replacement rule)

**ASSUMPTION MADE:** Tests assume Option B (replacement rule) - simpler interpretation

**QUESTION:** For 14 players with communists, is it 4 emergency powers (1×4) or 6 emergency powers (1×4 + 2×1)?

### 4. Communist Knowledge Rules Timing (Lines 31-32 in test.md vs Lines 154-159 in rules.pdf)

**test.md states:** "Communists know each other at start if 11+ players"

**rules.pdf states:** "If there are 11 players or more the communists know who the other communists are at the start."

**AMBIGUITY:** What happens when players are radicalized during the game? Do the knowledge rules change?

**ASSUMPTION MADE:** Tests assume knowledge rules are determined at game start and don't change

**QUESTION:** If a game starts with <11 players but players get radicalized, do communists learn each other's identities?

### 5. Anti-Policy Placement Ambiguity (Lines 111-126 in test.md vs Lines 538-573 in rules.pdf)

**test.md states:** Anti-policies are "placed on" specific trackers

**rules.pdf states:** Anti-policies are "placed on" trackers but then "remove" policies from other trackers

**AMBIGUITY:** Do anti-policies physically occupy tracker slots, or are they just temporary effects?

**ASSUMPTION MADE:** Tests assume anti-policies occupy tracker slots like normal policies

**QUESTION:** Do anti-policies count toward victory conditions, or are they just removal mechanisms?

---

## 🟢 MINOR CLARIFICATIONS NEEDED

### 6. Policy Deck Ratios for 17-20 Players

**MISSING:** Rules.pdf doesn't specify policy deck compositions for player counts above 16

**ASSUMPTION MADE:** Tests extrapolate from existing patterns, but this is speculative

**QUESTION:** What are the exact policy deck ratios for 17-20 players?

### 7. Communist Tracker Variations (Lines 76-79 in test.md vs Lines 327-348 in rules.pdf)

**MINOR DISCREPANCY:** test.md mentions "blank" slot for 11+ players, rules.pdf says "blank" slot

**ASSUMPTION MADE:** Tests assume "blank" means no power activation

**QUESTION:** Does "blank" slot still count toward communist victory, or is it truly empty?

### 8. Emergency Power Card Types Distribution

**AMBIGUITY:** Rules specify maximum 6 emergency powers (3 Article 48, 3 Enabling Act) but don't specify distribution for smaller counts

**ASSUMPTION MADE:** Tests assume even distribution when possible

**QUESTION:** For 2 emergency powers, is it 1 Article 48 + 1 Enabling Act, or player's choice?

---

## 🔵 IMPLEMENTATION-SPECIFIC QUESTIONS

### 9. Digital "Eyes Closed" Phases

**CHALLENGE:** Rules.pdf describes physical "eyes closed" mechanics (Lines 232-242, 261-267, 282-310)

**ASSUMPTION MADE:** Tests assume digital implementation handles secret phases appropriately

**QUESTION:** How should digital implementation handle simultaneous secret actions by multiple communists?

### 10. Card Shuffling Simulation

**CHALLENGE:** Rules mention physical card shuffling and placement (Lines 241, 309)

**ASSUMPTION MADE:** Tests assume digital shuffling provides equivalent randomization

**QUESTION:** Should digital implementation simulate the "move your card around slightly" mechanic?

---

## 📋 RECOMMENDED TEST APPROACH

Given these ambiguities, the tests have been written with the following strategy:

1. **Follow test.md specifications** where they align with rules.pdf
2. **Make conservative assumptions** where ambiguities exist
3. **Document assumptions clearly** in test comments
4. **Allow tests to fail** where implementation differs from assumptions
5. **Highlight discrepancies** for future clarification

### Test Comments Strategy

Each ambiguous test includes comments like:
```rust
// AMBIGUITY: Rules.pdf unclear on player counts 17-20
// ASSUMPTION: Extrapolating from existing patterns
// QUESTION: Need official ratios for these player counts
```

### Priority for Clarification

1. **HIGH PRIORITY:** Player count limits and ratios (affects core gameplay)
2. **MEDIUM PRIORITY:** Emergency power distribution rules (affects balance)
3. **LOW PRIORITY:** Role name consistency (cosmetic issue)

---

## 🎯 CONCLUSION

The test.md checklist is generally well-aligned with the rules.pdf, but several ambiguities exist that could lead to implementation differences. The tests have been written to be comprehensive while documenting assumptions clearly. When the implementation differs from test expectations, it may indicate either:

1. A bug in the implementation
2. A different interpretation of ambiguous rules
3. An incomplete specification in test.md

Each failing test should be evaluated against this analysis to determine the appropriate resolution.