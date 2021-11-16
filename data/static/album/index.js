let pop = document.querySelectorAll('.pop')[0];

fetch(`${window.location.pathname}/photos`)
  .then((r) => r.json())
  .then((r) =>
    r.forEach((item, i) => {
      let img = document.createElement("img");
      img.src = `${window.location.pathname}/photo/${item}`;
      img.classList.add("img");

      img.addEventListener('click', () => {
        pop.innerHTML = img.outerHTML;
        pop.style.opacity = 1;
        pop.style.pointerEvents = 'all';
        document.body.style.overflow = 'hidden';
      })

      document.querySelectorAll(".photos")[0].appendChild(img);
    })
  );

pop.addEventListener('click', () => {
  pop.style.opacity = 0;
  pop.style.pointerEvents = 'none';
  document.body.style.overflow = 'auto';
  document.body.style.overflowX = 'hidden';
});
