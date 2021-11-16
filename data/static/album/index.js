let pop = document.querySelectorAll(".pop")[0];
// let images = [];

fetch(`${window.location.pathname}/photos`)
  .then((r) => r.json())
  .then((r) =>
    r.forEach((item, i) => {
      images.push(`${window.location.pathname}/photo/${item}`);

      let img = document.createElement("img");
      img.src = `${window.location.pathname}/photo/${item}`;
      img.classList.add("img");

      img.addEventListener("click", () => {
        pop.innerHTML = img.outerHTML;
        pop.style.opacity = 1;
        pop.style.pointerEvents = "all";

        if (pop.clientHeight / pop.clientWidth >= 1)
          pop.children[0].style.width = "85vw";
        else pop.children[0].style.height = "85vh";

        document.body.style.overflow = "hidden";
      });

      document.querySelectorAll(".photos")[0].appendChild(img);
    })
  );

pop.addEventListener("click", () => {
  pop.style.opacity = 0;
  pop.style.pointerEvents = "none";
  document.body.style.overflow = "auto";
  document.body.style.overflowX = "hidden";
});

// window.addEventListener("keydown", (e) => {
//   if (e.key === "ArrowLeft") {
//   }
//
//   if (e.key === "ArrowRight") {
//   }
// });
