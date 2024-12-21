var last_card_id = 0;
const card_delay = 150; // Milliseconds

const base_card = document.getElementById("base_card");
const page_games_list = document.getElementById("page-games-list");

function add_game(name, image) {
  const new_card = base_card.cloneNode(true);
  new_card.id="card_" + last_card_id;
  new_card.style = "";
  new_card.getElementsByClassName("card-name")[0].innerText = name;
  new_card.getElementsByClassName("game-img")[0].src = image;
  new_card.style = "animation-delay: " + (800 + last_card_id*card_delay) + "ms"
  page_games_list.appendChild(new_card);

  new_card.addEventListener("click", ()=> select_game(new_card));


  last_card_id++;
}

function select_game(card) {
  const x_offset = card.getBoundingClientRect().x - document.body.getBoundingClientRect().x;

  console.log(x_offset)

  card.style = "position: absolute; transform: translate(" + x_offset + "px, 0px); scale: 1.5;";
}

document.addEventListener("DOMContentLoaded", () => {
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
  add_game("Hello Charlotte", "assets/game_img.jpg");
});