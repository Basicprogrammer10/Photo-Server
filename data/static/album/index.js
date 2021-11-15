fetch(`${window.location.pathname}/photos`)
  .then((r) => r.json())
  .then((r) =>
    r.forEach((item) => {
      let img = document.createElement("img");
      img.src = `${window.location.pathname}/photo/${item}`;
      img.classList.add('img');
      document.querySelectorAll(".photos")[0].appendChild(img);
    })
  );
