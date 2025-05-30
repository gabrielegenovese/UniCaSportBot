# UniCa Sport Bot

A Telegram bot that notifies users when new events are published on the
[Université Côte d'Azur sport website](https://sport.univ-cotedazur.fr/fr/).

## Usage

1. **Clone the repository**

2. **Set the Telegram bot token**

   Create a `.env` file or export the token:

   ```bash
   export TELOXIDE_TOKEN=your_telegram_bot_token
   export UNICABOT_DATA_DIR=chose_a_directory
   ```

3. **Run the bot**

   ```bash
   cargo run
   ```

## Commands

- `/help` – Show available commands
- `/subscribe` – Start receiving notifications
- `/unsubscribe` – Stop receiving notifications
- `/events` – View the current list of known events
- `/amisubscribed` - Check if you're already subscribed to the notification
