const { invoke } = window.__TAURI__.core;


var last_card_id = 0;
const card_delay = 150; // Milliseconds

const base_card = document.getElementById("base_card");
const page_games_list = document.getElementById("page-games-list");
const steam_scan_button = document.getElementById("steam-scan-button");
const base_game_found_list_element = document.getElementById("base-game-found-list-element");
const steam_scan_result_list = document.getElementById("steam-scan-result-list");

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
    var base_card_fake = document.getElementById("base_card_fake");
    var br = new_card.getBoundingClientRect();
    base_card_fake.style.display = "flex";
    base_card_fake.classList = "game-card game-card-fake";
    base_card_fake.style.position = 'fixed';
    base_card_fake.style.left = (br.left - 10) + 'px';
    base_card_fake.style.top = br.top + 'px';
    base_card_fake.style.scale = 1;

    setTimeout(() => {
      page_games_list.style.height = 0;
      base_card_fake.style.overflowY = "scroll";
    }, 1000) // hide all cards
  });

  last_card_id++;
}

function dispawn_game_page() {
  let cards = document.getElementsByClassName("game-card");

  for (let i = 0; i < cards.length; i++) {
    cards[i].classList = "game-card game-card-dispawned";
  }

  document.getElementById("add-game-button").style.animation = "dispawn-add-game-button 1s ease 0.2s forwards";

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
        element_copy.style.animationDelay = i*200 + "ms";
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

document.addEventListener("DOMContentLoaded", () => {
  document.getElementById("add-game-button").addEventListener("click", dispawn_game_page);
  document.getElementById("steam-scan-button").addEventListener("click", steam_scan_clicked);

  invoke("get_remote_available_patches").then((message) => {
    console.log(message);
    for (let i=0; i<message.length; i++) {
      add_game(message[i]["name"], TM_BASE_URL+message[i]["card_image"]);
    }
  });
});