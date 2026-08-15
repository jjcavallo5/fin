<div align="center">
    <h1 align="center"><i>FIN</i></h1>
    <div align="center">
        <a align="center" href='https://jeremycavallo.com/blog'><img src="https://img.shields.io/badge/Blog-white?logo=ghostty&logoColor=blue" alt="blog"></a>
        <a align="center" href='https://github.com/jjcavallo5/the-art-of-code'><img src="https://img.shields.io/github/stars/jjcavallo5/the-art-of-code" alt="stars"></a>
    </div>
    <p align="center"><i>Financial Automation CLI</i></h1>
</div>

<div align="center">
    <img width="1024" height="572" alt="image" src="https://github.com/user-attachments/assets/b2f08bf9-b7fd-4154-bab3-bd07ae937d17" />
</div>

<br>

## Installation

Clone the repo, then build the binary:

```bash
cargo build --release
```

Then add the binary to your path

## Setup

To begin, you'll need a Plaid developer account, which requires getting approved by the Plaid team. Since this is a single-user local application, usually this isn't an issue.

Once you have your account, you'll need the following products:

- Transactions
- Investments
- Liabilities

You'll also need to be able to access your keys, which you will be prompted for when you first log in to Fin.

## Getting Started

To start, you'll first need to log into Fin:

```bash
fin login
```

This will prompt you for a password - remember this password. It will need to be the same every time you log in. Otherwise, you won't be able to decrypt your data and you'll need to re-link all of your accounts.

Then, paste your Plaid Client ID and production secret.

> [!NOTE]
> The Plaid Client ID and Secret are stored ephemerally in memory while the Fin daemon is running. They are never written to disk. Be sure to run the `fin quit` (or `fin stop`) commands once your session is complete to stop the daemon.

## Link an Account

To link an account, you'll need to first decide which account you'd like to link:

```bash
# For banks
fin link bank

# For investment accounts, retirement accounts, etc.
fin link investement

# For credit cards, mortgages, etc.
fin link liability
```

These commands will open a browser and the Plaid linking process will begin. Find your account in the window and log in. When login is successful, the account will be linked to your Fin account.

## Unlink an Account

To unlink an account, run `fin unlink`. This will open a menu where you can select which account to unlink. Use `j` and `k` to navigate the menu (because Vim motions are superior).

## Create and Run a Plan

A plan describes how Fin should preserve minimum account balances, pay liabilities, and allocate excess money between accounts. Log in and link at least one asset account before creating one; liability rules also require the relevant liability accounts to be linked.

Create a plan with the interactive setup:

```bash
fin plan create
```

Fin will ask for a name, optional minimum balances, liability payment rules, and optional excess-allocation rules. Review the summary and confirm that you want to save it. The success message includes the plan ID.

Run the saved plan by passing that ID:

```bash
fin plan execute <PLAN_ID>
```

Fin evaluates the plan against current account balances and prints the recommended payments and transfers, along with the expected final balances. It reports any required rules that cannot be satisfied.

> [!IMPORTANT]
> Fin does NOT initiate any transactions or more money in any way. It simply tells you how to move money to satisify a plan's rules.

## Commands

- `fin balance` — Show the current and available balances for all linked accounts.
- `fin daemon` — Start the local Fin daemon in the foreground.
- `fin link bank` — Link a bank account through Plaid.
- `fin link investment` — Link an investment or retirement account through Plaid.
- `fin link liability` — Link a credit card, loan, mortgage, or other liability through Plaid.
- `fin login` — Start the daemon and authenticate with your encryption password and Plaid credentials.
- `fin list` — List the institutions currently linked to Fin.
- `fin ping` — Check whether the local Fin daemon is responding.
- `fin plan create` — Interactively create and save a financial plan.
- `fin plan execute <PLAN_ID>` — Evaluate a saved plan against current balances and print its recommended actions.
- `fin quit` — Stop the local Fin daemon.
- `fin stop` — Stop the local Fin daemon; this is equivalent to `fin quit`.
- `fin unlink` — Select and remove a linked institution.
- `fin net-worth` — Calculate net worth from all linked asset and liability balances; `fin nw` is an alias.
- `fin help` — Show the command list; use `fin <COMMAND> --help` for help with a specific command.
