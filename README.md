<h1 align="center">Horizon Offline Launcher</h1>

<p align="center">
  <b>Лаунчер для <a href="https://github.com/horizon-rust/horizon">Horizon</a> (Minecraft: Java Edition 26.3, переписанный на Rust) без обязательного аккаунта Microsoft и с выбором своего jar.</b><br>
  A launcher for Horizon (Minecraft: Java Edition 26.3 rewritten in Rust) that plays without a Microsoft account and lets you pick your own jar.
</p>

<p align="center">
  <a href="https://github.com/gerdaroot/horizon-offline-launcher/releases/latest"><b>Скачать / Download</b></a> ·
  <a href="#установка">Установка</a> ·
  <a href="#первый-запуск">Первый запуск</a> ·
  <a href="#вопросы">Вопросы</a> ·
  <a href="#english">English</a>
</p>

<p align="center">
  <a href="https://github.com/gerdaroot/horizon-offline-launcher/actions/workflows/launcher.yml"><img src="https://github.com/gerdaroot/horizon-offline-launcher/actions/workflows/launcher.yml/badge.svg" alt="Build"></a>
  <img src="https://img.shields.io/badge/Minecraft-Java%2026.3-5D9C3F" alt="Minecraft Java 26.3">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-2A2470" alt="Windows, Linux, macOS">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-3B2F7E" alt="MIT"></a>
</p>

<p align="center">
  <img src="docs/media/launcher-play.webp" alt="Главная страница лаунчера" width="80%">
</p>

## Что это

[Horizon](https://github.com/horizon-rust/horizon) — это Minecraft: Java Edition 26.3, заново написанный на Rust: свои рендер, мир, мобы и редстоун, совместимые с ванильными мирами. Сама игра берётся из официальных релизов Horizon. Этот репозиторий — форк его лаунчера, в котором:

- **Аккаунт не нужен.** «Играть» запускает игру офлайн под вашим ником. Аккаунт Microsoft можно добавить, если нужны онлайн-серверы с лицензионным ником и скином.
- **Можно выбрать свой jar.** Кнопка «Выбрать jar…» берёт файл `26.3.jar` откуда угодно: из TLauncher, папки «Загрузки» или с флешки. Без выбора лаунчер сам ищет его в папках официального лаунчера, TLauncher, Prism, MultiMC, Modrinth, ATLauncher и GDLauncher.
- **Лаунчер не обновляется сам.** Иначе официальное обновление вернуло бы обязательный вход.
- **Установка в одну команду.** Ярлыки на рабочем столе и в меню «Пуск» создаются сами.

Всё остальное — как в Horizon: моды, ресурс-паки, сборки, серверы, игра с друзьями по коду.

> Неофициальный проект, не связан с Mojang, Microsoft и авторами Horizon. Файлов Minecraft здесь нет: игре нужен ваш собственный Minecraft: Java Edition 26.3.

## Установка

### Windows

Откройте PowerShell (Win+X → «Терминал») и вставьте:

```powershell
irm https://github.com/gerdaroot/horizon-offline-launcher/releases/latest/download/install.ps1 | iex
```

Скрипт скачает лаунчер в `%LOCALAPPDATA%\Programs\HorizonOfflineLauncher`, создаст ярлыки «Horizon Offline Launcher» и запустит его. Права администратора не нужны.

Вручную: скачайте `horizon-offline-launcher-windows-x64.exe` со [страницы релизов](https://github.com/gerdaroot/horizon-offline-launcher/releases/latest) и запустите. Если Windows покажет «Windows защитила ваш компьютер», нажмите «Подробнее» → «Выполнить в любом случае»: файл просто не подписан сертификатом.

### Linux и macOS

```sh
curl -fsSL https://github.com/gerdaroot/horizon-offline-launcher/releases/latest/download/install.sh | sh
```

После установки лаунчер запускается командой `horizon-offline-launcher`. На Linux он сам добавит себя в меню приложений.

## Первый запуск

1. **Нужен Minecraft 26.3.** Если его ещё нет, скачайте и один раз запустите версию **26.3** в любом лаунчере (официальном, TLauncher, Prism и т. д.).
2. **Откройте Horizon Offline Launcher.** Он сам скачает последнюю версию Horizon.
3. **Нажмите «Подготовить»** в строке «Ресурсы». Лаунчер найдёт ваш `26.3.jar` и соберёт из него текстуры и звуки. Это делается один раз и занимает несколько минут. Если jar не нашёлся, нажмите **«Выбрать jar…»** и укажите файл `26.3.jar`; обычно он лежит в `.minecraft\versions\26.3\`.
4. **Нажмите «Играть».**

Ник для офлайн-игры меняется вверху справа, кнопкой с именем игрока.

## Вопросы

**Подойдёт jar другой версии, например 1.21?** Нет. Horizon воспроизводит именно 26.3, его данные и сетевой протокол привязаны к этой версии. Лаунчер проверяет контрольную сумму файла и, если jar не тот, прямо говорит об этом.

**«Подготовка» просит Java.** Для подготовки нужна Java 25. Лаунчер ищет её у других лаунчеров и, если не находит, скачивает сам. Если не вышло, установите [Java 25](https://adoptium.net/temurin/releases/?version=25) и повторите.

**Как играть с друзьями?** Пауза → «Друзья» даёт код приглашения, LAN тоже работает. Для онлайн-серверов (online-mode) нужен аккаунт Microsoft: он добавляется в настройках, в разделе «Аккаунты».

**Где мои миры?** Там же, где у официального лаунчера Horizon: обе программы используют одну папку данных, так что миры, сборки и настройки общие.

**Как обновиться?** Запустите команду установки ещё раз: она поставит последний релиз поверх старого. Миры и настройки останутся на месте.

## Сборка из исходников

Windows (нужны [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) с «Разработкой классических приложений на C++»):

```powershell
git clone https://github.com/gerdaroot/horizon-offline-launcher
cd horizon-offline-launcher
powershell -ExecutionPolicy Bypass -File build.ps1
```

`build.ps1` сам поставит Rust, если его нет, найдёт компилятор шейдеров `fxc.exe` из Windows SDK на любом диске, соберёт лаунчер и установит его с ярлыками.

Linux и macOS:

```sh
cd launcher
cargo build --release
./target/release/horizon-launcher
```

На Linux перед сборкой нужны пакеты `libxkbcommon-x11-dev libwayland-dev libx11-xcb-dev libfontconfig-dev libvulkan-dev libasound2-dev`.

Релиз выпускается тегом: `git tag v1.0.0 && git push --tags`. GitHub Actions соберёт Windows, Linux и macOS и выложит файлы вместе со скриптами установки.

## Чем отличается от Horizon

| | Horizon | Horizon Offline Launcher |
|---|---|---|
| Аккаунт Microsoft | обязателен | по желанию |
| Свой jar-файл | нет, только папки лаунчеров | да, кнопка «Выбрать jar…» |
| Обновление лаунчера | само | командой установки |
| Игра | релизы Horizon | те же релизы Horizon |

Изменения лаунчера: `launcher/src/account.rs` (вход больше не обязателен), `launcher/src/sources.rs` (`from_jar`), `launcher/src/main.rs` (интерфейс), `launcher/src/backend.rs`. Код игры (`crates/`) не изменён.

## Благодарности и лицензия

Игра и почти весь код — [Horizon](https://github.com/horizon-rust/horizon) от Ludovic Marie и участников проекта. Лицензия MIT, см. [LICENSE](LICENSE). Подробная документация по игре: [playhorizon.dev/docs](https://playhorizon.dev/docs/).

---

## English

**Horizon Offline Launcher** is a fork of the [Horizon](https://github.com/horizon-rust/horizon) launcher (Horizon is Minecraft: Java Edition 26.3 rewritten in Rust). It still installs the official Horizon game releases, with these changes:

- **No account required.** Play starts the game offline under your own name; a Microsoft account is optional, for online-mode servers.
- **Pick your own jar.** "Choose jar…" takes a `26.3.jar` from anywhere; otherwise the launcher finds it in the official launcher's, TLauncher's, Prism's, MultiMC's, Modrinth's, ATLauncher's or GDLauncher's folder.
- **No self-update**, so an upstream update can't bring the account requirement back. Run the install command again to update.

**Install.** Windows (PowerShell):

```powershell
irm https://github.com/gerdaroot/horizon-offline-launcher/releases/latest/download/install.ps1 | iex
```

Linux / macOS:

```sh
curl -fsSL https://github.com/gerdaroot/horizon-offline-launcher/releases/latest/download/install.sh | sh
```

**First run.** Have Minecraft 26.3 downloaded by any launcher, open Horizon Offline Launcher, press **Prepare** (or **Choose jar…** first), then **Play**. Only the 26.3 release jar works: Horizon is built for that exact version, and the launcher checks the file's checksum.

Unofficial; not affiliated with Mojang, Microsoft or the Horizon authors. No Minecraft files are included: you need your own copy of Minecraft: Java Edition 26.3. MIT license.
