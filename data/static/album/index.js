// const cover = document.querySelectorAll(".cover")[0];
// const readme = document.querySelectorAll(".readme")[0];
// let withHeight = false;
//
// new ResizeObserver(() => {
//   console.log(`${readme.clientWidth}, ${cover.width}`);
//   if (readme.clientWidth > cover.width && !withHeight) {
//     cover.style.width = "100vw";
//     cover.style.height = "";
//     withHeight = true;
//     return;
//   }
//
//   cover.style.width = "";
//   cover.style.height = "100%";
//   withHeight = false;
// }).observe(readme);

fetch(`${window.location.pathname}/photos`)
  .then((r) => r.json())
  .then((r) =>
    r.forEach((item, i) => {
      let img = document.createElement("img");
      img.src = `${window.location.pathname}/photo/${item}`;
      img.classList.add("img");
      document.querySelectorAll(".photos")[0].appendChild(img);
    })
  );
