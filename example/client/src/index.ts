import { client } from "./client";
import { ZetroQuery, ZetroMutation } from "./generated/code_generated";

const roomListDiv: HTMLDivElement = document.querySelector("#room-list");
const messageListDiv: HTMLDivElement = document.querySelector("#message-list");
const usernameInput: HTMLInputElement = document.querySelector("#username");
const textBoxInput: HTMLInputElement = document.querySelector("#textbox input");
const textBoxSendBtn: HTMLButtonElement =
  document.querySelector("#textbox button");

textBoxSendBtn.addEventListener("click", sendMessage);

let currentRoom: number = null;

(async function main() {
  await refreshRooms();
})();

async function refreshRooms() {
  const { getRooms } = await new ZetroQuery(client).getRooms({}).fetch();

  roomListDiv.innerHTML = "";
  for (const room of getRooms.rooms) {
    const roomRow = document.createElement("div");
    const name = document.createElement("p");
    name.innerHTML = `<b>${room.name}</b> (${room.messages.length} messages)`;
    roomRow.appendChild(name);

    roomRow.addEventListener("click", () => {
      currentRoom = room.id;
      refreshRooms();
    });

    roomListDiv.appendChild(roomRow);
  }

  if (currentRoom == null) {
    currentRoom = getRooms.rooms[0].id;
  }

  // Show messages in selected room
  messageListDiv.innerHTML = "";

  const selectedRoom = getRooms.rooms.find((room) => room.id == currentRoom);
  for (const message of selectedRoom.messages) {
    const msgRow = document.createElement("div");
    msgRow.style.marginBottom = "8px";
    // Shouldn't use innerHTML but this is an example
    msgRow.innerHTML = `
      <b>${message.author.username}</b> <i>(${new Date(
      message.date * 1000
    ).toDateString()})</i>: ${message.text}
    `;

    messageListDiv.appendChild(msgRow);
  }
}

async function sendMessage() {
  if (currentRoom == null) return;
  if (usernameInput.value == "") {
    alert("Enter a username");
    return;
  }

  const message = textBoxInput.value.trim();
  if (message.length == 0) {
    return;
  }
  textBoxInput.value = "";

  const { sendMessage: mid } = await new ZetroMutation(client)
    .sendMessage({
      msg: {
        author: {
          username: usernameInput.value,
        },
        date: Math.floor(Date.now() / 1000),
        id: 0,
        text: message,
      },
      roomId: currentRoom!,
    })
    .fetch();

  await refreshRooms();
}
