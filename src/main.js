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

function add_game(name, image) {
  const new_card = base_card.cloneNode(true);
  new_card.id = "card_" + last_card_id;
  new_card.style = "";
  new_card.getElementsByClassName("card-name")[0].innerText = name;
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
    base_card_fake.style.animation = "card-zoom-transition 1s ease 0s forwards";

    setTimeout(() => {
      page_games_list.style.height = 0;
      base_card_fake.style.overflowY = "scroll";
    }, 1000) // hide all cards
  });

  last_card_id++;
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
        add_game(message[i]["name"], TM_BASE_URL + message[i]["card_image"]);
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

  reload_game_cards(true);

  invoke("loading_finished");
});
