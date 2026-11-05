# Ozone Pokemon Bot - Agent Guidelines & Mandatory Policies

## 1. Custom Emoji Arrow Policy (STRICT - ALL PRESENT & FUTURE FEATURES)
The user has established a mandatory global rule: **ALWAYS** use the user's custom green box arrows for all navigation, directional arrows, and pagination controls:

- **Left Arrow (`←`)**:
  - **Custom Emoji ID**: `5283240108328244731`
  - **Constant**: `LEFT_ARROW_CUSTOM_EMOJI_ID`
  - **HTML**: `<tg-emoji emoji-id="5283240108328244731">←</tg-emoji>` (`LEFT_ARROW_TG_EMOJI`)
- **Right Arrow (`→`)**:
  - **Custom Emoji ID**: `5282981997973621943`
  - **Constant**: `RIGHT_ARROW_CUSTOM_EMOJI_ID`
  - **HTML**: `<tg-emoji emoji-id="5282981997973621943">→</tg-emoji>` (`RIGHT_ARROW_TG_EMOJI`)

### Keyboard Implementation:
- Row 1 navigation buttons in inline keyboards must contain:
  - Left button: `"icon_custom_emoji_id": LEFT_ARROW_CUSTOM_EMOJI_ID` (`5283240108328244731`)
  - Right button: `"icon_custom_emoji_id": RIGHT_ARROW_CUSTOM_EMOJI_ID` (`5282981997973621943`)
- Send via raw Telegram Bot API JSON (`sendMessage` / `editMessageText`) using `bot.client().post(&endpoint).json(&payload).send().await` since Teloxide's Rust struct lacks `icon_custom_emoji_id`.
- **NEVER** replace or revert these custom arrow emojis with standard Unicode arrows (`⬅️`, `➡️`).

## 2. Strict Emoji Discipline Policy (DO NOT USE EXTRA EMOJIS)
- **Zero Unrequested Emojis**: **NEVER** add decorative, unprompted, or extraneous emojis (such as `✅`, `❌`, `🎉`, `✨`, `🚨`, `⚠️`, `💡`, `🔥`, `👍`, `📌`, etc.) to bot messages, commands, confirmations, popups, or button text unless the user explicitly asks for them.
- **Clean & Minimal UI**: Keep messages and alerts concise, clean, and professional. Do not add checkmarks or "success" badges to command responses unless specifically instructed by the user.
- **Preserve Only Established Game Emojis**: Only use existing canonical game assets (e.g., PokéDollars `💵`, specific Poké Ball / type icons where defined in existing code) and the mandatory custom green box arrows (`LEFT_ARROW_CUSTOM_EMOJI_ID`, `RIGHT_ARROW_CUSTOM_EMOJI_ID`). Do not introduce new or extra emojis on your own initiative.

