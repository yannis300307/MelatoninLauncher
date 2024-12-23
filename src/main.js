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

  new_card.addEventListener('click', () => {
    var base_card_fake = document.getElementById("base_card_fake");
    var br= new_card.getBoundingClientRect();
    base_card_fake.style.display = "flex";
    base_card_fake.classList = "game-card game-card-fake";
    base_card_fake.style.position= 'fixed';
    base_card_fake.style.left= (br.left-10)+'px';
    base_card_fake.style.top = br.top +'px';
    base_card_fake.style.scale = 1;

    setTimeout(() => {
      page_games_list.style.height = 0;
      base_card_fake.style.overflowY = "scroll";
    }, 1000) // hide all sprites

    //page_games_list.appendChild(base_card_fake);
  });

  last_card_id++;
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