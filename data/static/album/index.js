const pop = document.querySelectorAll(".pop")[0];
const downloadButtons = document.querySelectorAll(".downloadButton");
let images = [];

fetch(`${window.location.pathname}/photos`)
  .then((r) => r.json())
  .then((r) =>
    r.forEach((item) => {
      images.push(`${window.location.pathname}/thumb/${item}`);

      let div = document.createElement("div");

      let icon = document.createElement("i");
      icon.classList.add("fa");
      icon.classList.add("fa-download");
      icon.classList.add("downloadButton");

      let img = document.createElement("img");
      img.src = `${window.location.pathname}/thumb/${item}`;
      img.classList.add("img");

      icon.addEventListener("click", () => {
        download(item, `${window.location.pathname}/photo/${item}`);
      });

      img.addEventListener("mouseover", () => {
        icon.style.opacity = 1;
        icon.style.pointerEvents = "auto";
        icon.style.transform = "translate(-25px, 240px)";
      });

      img.addEventListener("mouseout", () => {
        icon.style.opacity = 0;
        icon.style.pointerEvents = "none";
        icon.style.transform = "translate(-30px, 235px)";
      });

      img.addEventListener("click", () => {
        let img = document.createElement("img");
        img.src = `${window.location.pathname}/photo/${item}`;
        img.classList.add("img");

        pop.innerHTML = img.outerHTML;
        pop.style.opacity = 1;
        pop.style.pointerEvents = "all";
        pop.children[0].style.transform = 'translate(-50%, -50%) scale(1)'

        if (pop.clientHeight / pop.clientWidth >= 1)
          pop.children[0].style.width = "85vw";
        else pop.children[0].style.height = "85vh";

        document.body.style.overflow = "hidden";
      });

      div.appendChild(img);
      div.appendChild(icon);
      document.querySelectorAll(".photos")[0].appendChild(div);
    })
  );

pop.addEventListener("click", () => {
  pop.style.opacity = 0;
  pop.style.pointerEvents = "none";
  pop.children[0].style.transform = 'translate(-50%, -50%) scale(0.9)'
  document.body.style.overflow = "auto";
  document.body.style.overflowX = "hidden";
});

function download(filename, url) {
  let element = document.createElement("a");
  element.setAttribute("href", url);
  element.setAttribute("download", filename);

  element.style.display = "none";
  document.body.appendChild(element);

  element.click();

  document.body.removeChild(element);
}
