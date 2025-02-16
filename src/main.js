const { invoke } = window.__TAURI__.core;


var last_card_id = 0;
const card_delay = 150; // Milliseconds

const base_card = document.getElementById("base_card");
const page_games_list = document.getElementById("page-games-list");
const steam_scan_button = document.getElementById("steam-scan-button");
const base_game_found_list_element = document.getElementById("base-game-found-list-element");
const steam_scan_result_list = document.getElementById("steam-scan-result-list");
const notif_container = document.getElementById("notif-container");

const TM_BASE_URL = "https://team-melatonin.fr/";

var found_apps = [];
var steam_scan_loading = false;

var theme = "light";

function add_game(info) {
  let image = TM_BASE_URL + info["card_image"];

  const new_card = base_card.cloneNode(true);
  new_card.id = "card_" + last_card_id;
  new_card.style = "";
  new_card.getElementsByClassName("card-name")[0].innerText = info["name"];
  new_card.getElementsByClassName("game-img")[0].src = image;
  new_card.style = "animation-delay: " + (800 + last_card_id * card_delay) + "ms"
  page_games_list.appendChild(new_card);

  new_card.addEventListener('click', () => {
    document.getElementById("reload-games-button").style.animation = "dispawn-reload-games-button 1s ease 0.2s forwards";
    var base_card_fake = document.getElementById("base_card_fake");
    var br = new_card.getBoundingClientRect();
    base_card_fake.style.display = "flex";
    base_card_fake.classList = "game-card game-card-fake";
    base_card_fake.style.position = 'fixed';
    base_card_fake.style.left = (br.left - 10) + 'px';
    base_card_fake.style.top = br.top + 'px';
    base_card_fake.style.scale = 1;
    base_card_fake.getElementsByClassName("game-img")[0].src = image;
    base_card_fake.querySelector("#game-version-label").innerText = "version du patch : " + info["patch_version"];
    base_card_fake.style.animation = "card-zoom-transition 1s ease 0s forwards";

    let add_from_steam_game_button = base_card_fake.querySelector("#add-from-steam-game-button");
    let manual_add_game_button = base_card_fake.querySelector("#manual-add-game-button");
    let enable_patch_game_button = base_card_fake.querySelector("#enable-patch-game-button");
    let disable_patch_game_button = base_card_fake.querySelector("#disable-patch-game-button");
    let game_scan_info_label = base_card_fake.querySelector("#game-scan-info-label");

    add_from_steam_game_button.style.display = "";
    manual_add_game_button.style.display = "";
    enable_patch_game_button.style.display = "";
    disable_patch_game_button.style.display = "";

    if (info["registered"]) {
      add_from_steam_game_button.style.display = "none";
      manual_add_game_button.style.display = "none";

      if (info["patch_activated"]) enable_patch_game_button.style.display = "none";
      else disable_patch_game_button.style.display = "none";

    } else {
      if (!info["installed_on_steam"]) {
        add_from_steam_game_button.style.display = "none";
      }
      enable_patch_game_button.style.display = "none";
      disable_patch_game_button.style.display = "none";
    }

    enable_patch_game_button.addEventListener("click", () => {
      invoke("enable_patch", { globalId: info["global_id"] }).then(() => {

      }).catch((msg) => {
        console.error(msg)
      });
    });

    add_from_steam_game_button.addEventListener("click", () => {
      add_from_steam_game_button.classList.add("button-loading");
      add_from_steam_game_button.style.animation = "button-to-loading 1s ease forwards, rotate 5s infinite 500ms linear";
      manual_add_game_button.style.animation = "dispawn-big-button 500ms ease forwards";

      setTimeout(() => {
        let got_error = false;
        invoke("register_app_from_steam", { globalId: info["global_id"] }).catch((error) => {
          console.error(error);
          game_scan_info_label.style.display = "inline";
          game_scan_info_label.innerText = error;
          add_from_steam_game_button.style.animation = "button-to-loading 1s ease forwards, rotate 5s infinite 500ms linear, move-up-infinite-loading-button 6s ease";
          game_scan_info_label.style.animation = "show-game-scan-info-label 6s alternate";
          got_error = true;
        }).then(
          () => {
            if (got_error) setTimeout(() => {
              add_from_steam_game_button.style.animation = "button-to-loading-reversed 1s ease forwards";
              manual_add_game_button.style.animation = "dispawn-big-button-reversed 500ms ease forwards";
              game_scan_info_label.style.display = "none";
            }, 6000);
            else {
              enable_patch_game_button.style.animation = "dispawn-big-button-reversed 500ms ease forwards";
              enable_patch_game_button.style.animation = "button-to-loading 0s linear forwards";
              enable_patch_game_button.style.display = ""
              enable_patch_game_button.style.animation = "button-to-loading-reversed 1s ease forwards";
              add_from_steam_game_button.style.display = "none";
            }
          }

        );
      }, 1000) // hide all cards
    });

    manual_add_game_button.addEventListener("click", () => {
      manual_add_game_button.classList.add("button-loading");
      manual_add_game_button.style.animation = "button-to-loading 1s ease forwards, rotate 5s infinite 500ms linear";
      add_from_steam_game_button.style.animation = "dispawn-big-button 500ms ease forwards";

      setTimeout(() => {
        manual_add_game_button.style.animation = "button-to-loading-reversed 1s ease forwards";
        add_from_steam_game_button.style.animation = "dispawn-big-button-reversed 500ms ease forwards";
      }, 1000) // hide all cards
    });


    setTimeout(() => {
      page_games_list.style.height = 0;
      base_card_fake.style.overflowY = "hidden";
    }, 1000) // hide all cards
  });

  last_card_id++;
}

function set_theme(name) {
  theme = name;
  if (name === "light") {
    document.documentElement.style.setProperty('--color1', '#fdf3de');
    document.documentElement.style.setProperty('--color2', '#9c628b');
    document.documentElement.style.setProperty('--color3', '#555555bf');
    document.documentElement.style.setProperty('--color4', '#553546');
  } else if (name == "dark") {
    document.documentElement.style.setProperty('--color1', '#1B1833');
    document.documentElement.style.setProperty('--color2', '#441752');
    document.documentElement.style.setProperty('--color3', '#AB4459');
    document.documentElement.style.setProperty('--color4', '#F29F58');
  }
}

function switchTheme() {
  if (theme === "dark") {
    set_theme("light");
  } else if (theme === "light") {
    set_theme("dark");
  }
}

function show_notif(text, delay) {
  notif_container.innerText = text;
  notif_container.style.transform = "translateY(calc(100vh - " + Math.max(notif_container.getBoundingClientRect().height + 20) + "px))";
  notif_container.style.opacity = "1";
  console.log("translateY(100vh - " + Math.max(notif_container.getBoundingClientRect().height) + "px)");
  setTimeout(() => {
    notif_container.style.transform = "";
    notif_container.style.opacity = "";
  }, delay);
}

function dispawn_game_page() {
  let cards = document.getElementsByClassName("game-card");

  for (let i = 0; i < cards.length; i++) {
    cards[i].classList = "game-card game-card-dispawned";
  }

  document.getElementById("reload-games-button").style.animation = "dispawn-reload-games-button 1s ease 0.2s forwards";

  setTimeout(() => { page_games_list.style.display = "none"; }, 1000);
}

function steam_scan_clicked() {
  if (steam_scan_loading) return;

  steam_scan_loading = true;

  steam_scan_button.classList = "button button-loading add-game-buttons";
  steam_scan_button.style.animation = "button-to-loading 1s ease forwards, rotate 5s infinite 500ms linear";

  setTimeout(() => {
    invoke('get_steam_installed_apps').then((message) => {
      console.log(message);
      for (let i = 0; i < message.length; i++) {
        let current_app = message[i];
        if (found_apps.includes(current_app["global_id"])) continue;

        found_apps.push(current_app["global_id"]);

        let element_copy = base_game_found_list_element.cloneNode(true);
        element_copy.id = "";
        element_copy.style.animationDelay = i * 200 + "ms";
        element_copy.getElementsByClassName("add-game-found-list-element-name")[0].innerText = current_app["name"];
        element_copy.getElementsByTagName("img")[0].src = TM_BASE_URL + current_app["icon"];
        steam_scan_result_list.appendChild(element_copy);
      }

      steam_scan_button.style.animation = "button-to-loading-reversed 1s ease forwards";
      steam_scan_button.classList = "button add-game-buttons";
      steam_scan_loading = false;
    }).catch((error) => console.error(error));
  }, 1000)
}

function reload_game_cards(first_load = false) {
  let cards = document.querySelectorAll(".game-card:not(#base_card):not(#base_card_fake)");

  if (!first_load) document.getElementById("reload-games-button").style.animation = "dispawn-reload-games-button 1s ease 0.2s forwards";

  for (let i = 0; i < cards.length; i++) {
    let current_card = cards[i];
    current_card.classList = "game-card game-card-dispawned";
  }

  setTimeout(() => {
    for (let i = 0; i < cards.length; i++) {
      page_games_list.removeChild(cards[i]);
    }

    invoke("get_remote_available_patches").then((message) => {
      console.log(message);
      for (let i = 0; i < message.length; i++) {
        add_game(message[i]);
      }
    }).catch((error) => {
      show_notif("Impossible de récupérer la liste des patchs. Veuillez vérifier votre connexion à Internet. Erreur du backend : " + error, 15000);
    });

    document.getElementById("reload-games-button").style.animation = "";
  }, 1000);
}

function close_game_page() {
  base_card_fake.style.overflowY = "hidden";
  base_card_fake.style.maxHeight = "0%";
  page_games_list.style.height = "100%";
  setTimeout(() => {
    base_card_fake.style.animation = 'none';
    base_card_fake.style.animation = "";
    base_card_fake.style = "display: none;";
  }, 1000);
}

document.addEventListener("DOMContentLoaded", () => {
  document.getElementById("reload-games-button").addEventListener("click", () => { reload_game_cards(false) });
  document.getElementById("close-game-page-button").addEventListener("click", close_game_page);
  document.getElementById("steam-scan-button").addEventListener("click", steam_scan_clicked);
  document.getElementById("switch-theme-button").addEventListener("click", switchTheme);

  reload_game_cards(true);

  invoke("loading_finished");

  set_theme("light");
});
