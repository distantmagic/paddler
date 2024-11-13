import "@hotwired/turbo";

document.addEventListener("turbo:before-cache", function () {
  for (const clicked of document.getElementsByClassName("turbo-clicked")) {
    clicked.classList.remove("turbo-clicked");
  }

  for (const form of document.querySelectorAll("form")) {
    form.reset();
  }

  for (const dialog of document.querySelectorAll("dialog")) {
    dialog.close();
  }
});

document.addEventListener("turbo:click", function (evt: Event) {
  const target = evt.target;

  if (target instanceof HTMLAnchorElement && !target.href.includes("#")) {
    target.classList.add("turbo-clicked");
  }

  if (target instanceof HTMLButtonElement) {
    target.classList.add("turbo-clicked");
  }
});
