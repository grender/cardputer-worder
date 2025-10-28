# Project Brief: CardWorder

CardWorder is an application designed for the Cardputer, a small ESP32-based computer equipped with a screen, keyboard, WiFi, and speaker. The main goal of CardWorder is to help users learn English words using spaced repetition algorithms, powered by the `rs-fsrs` crate. The application integrates NTP (Network Time Protocol) to ensure accurate timekeeping, which is essential for effective repetition scheduling.

## UI Design Approach

The UI is built around a minimalist, explicit pattern suitable for embedded devices with small screens and limited resources. Forms are constructed manually from primitive elements, with vertical stacking as the primary layout. Scrolling is supported for forms that exceed the screen height, using a compose step to dynamically calculate line heights and a viewport to render only visible content.
