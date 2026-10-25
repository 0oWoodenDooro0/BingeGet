---
status: accepted
---

# Channel Lock Architecture for Stream Consistency

Multi-source video aggregators provide video streams for each episode from disparate and fluctuating release channels (such as `ikzy` and `hnzy`), each differing in video bitrate, resolution, subtitle font/styling, and watermark placement. We decided to enforce a Channel Lock mechanism that binds all episode downloads of a series to a single selected channel, backed by a semi-automatic fallback mechanism when a channel lacks specific episodes. This guarantees uniform audio/video presentation across an entire series and avoids the jarring experience of alternating encodings between episodes.
