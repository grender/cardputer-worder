# Product Context

## Target Device
- Cardputer (ESP32-based small computer)
- Features: screen, keyboard, WiFi, sound speaker

## User Needs
- Efficient English vocabulary learning
- Automated, algorithm-driven repetition scheduling
- Accurate time synchronization for spaced repetition

## Product Goals
- Provide an intuitive interface for word learning and review
- Ensure reliable time updates via NTP for correct repetition intervals
- Leverage device hardware (screen, keyboard, WiFi, speaker) for an engaging learning experience

## UI/UX Constraints

- The device screen is small, so forms are scrollable.
- All UI is constructed from primitive elements for maximum efficiency and predictability.
- No high-level UI abstractions, builders, or event/callback systems are used.
- The user navigates forms using simple up/down controls, with smooth scrolling and clear focus.

## Device Specifications

- **Display:** 240 x 135 px color screen (landscape)
