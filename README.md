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
