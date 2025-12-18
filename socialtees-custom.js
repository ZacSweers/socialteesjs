!(function () {
    st = {
      $win: null,
      $document: null,
      $preFooter: null,
      $footer: null,

      pets: null,
      dogs: [],
      cats: [],
      other: [],
      $petsWrapper: null,
      $petsContent: null,
      $petsFilters: null,
      $petCount: null,
      $filterBtns: null,
      $loader: null,

      isLoading: true,
      numToLoad: 100,
      totalPets: 0,
      petsVisible: 0,
      petsToShow: 0,
      filter: null,

      // Data URL - loads from GitHub raw content
      dataURL: "https://raw.githubusercontent.com/ZacSweers/socialteesjs/main/data/pets.json",

      loaderURL:
        "https://static1.squarespace.com/static/572b597a1bbee0f4e8d01e5e/t/5766ebc23e00be24e54c7b08/1466362818303/spinner.gif",

      init: function () {
        st.$win = $(window);
        st.$document = $(document);
        st.$preFooter = $("#prefooter");
        st.$footer = $("#footer");
        st.$petsWrapper = $("#js-petfinder");

        if (st.$petsWrapper.length > 0) {
          // Add loader to page.
          st.$loader = $(st.getLoaderDOM());
          st.$petsWrapper.append(st.$loader);

          st.loadPets();

          st.$win.scroll(st.checkScroll);
        }
      },

      checkScroll: function () {
        const footerHeight = st.$preFooter.outerHeight() + st.$footer.outerHeight();

        if (st.petsVisible >= st.petsToShow || st.isLoading) {
          return false;
        } else if (
          st.$win.height() + st.$win.scrollTop() >=
          st.$document.height() - footerHeight
        ) {
          st.isLoading = true;
          st.$petsWrapper.append(st.$loader);
          st.addPets();
        }
      },

      loadPets: function () {
        fetch(st.dataURL)
          .then((response) => {
            if (!response.ok) {
              throw new Error("Failed to fetch pets data");
            }
            return response.json();
          })
          .then((data) => {
            st.pets = data.pets;
            st.totalPets = st.petsToShow = st.pets.length;
            st.processPets();
            st.setupDOM();
            st.$petsContent = $("#js-petfinder__content");
            st.$petsFilters = $("#js-petfinder__filters");
            st.addFilters();
            st.addPets();
          })
          .catch((err) => {
            console.error(err);
            st.$loader.remove();
            st.$petsWrapper.append(st.getErrorDOM());
          });
      },

      processPets: function () {
        if (st.pets === null) return;

        let i = 0;
        for (i; i < st.pets.length; i++) {
          if (st.pets[i].type === "Dog") {
            st.dogs.push(st.pets[i]);
          } else if (st.pets[i].type === "Cat") {
            st.cats.push(st.pets[i]);
          } else {
            st.other.push(st.pets[i]);
          }
        }
      },

      addPets: function () {
        if (st.petsToShow === 0) {
          st.$petsContent.append(st.getEmptyDOM());
        } else {
          st.$petsContent.append(st.getPetListingsDOM());
        }
        st.updatePetCount();

        window.setTimeout(function () {
          if (!st.$petsFilters.hasClass("loaded")) {
            st.$petsFilters.addClass("loaded");
          }
          st.$loader.remove();
          st.$petsContent.find(".col").each(function(index) {
            const $col = $(this);
            setTimeout(function() {
              $col.css({ opacity: 1, transform: "translateY(0)" });
            }, index * 50);
          });
          st.isLoading = false;
        }, 250);
      },

      getPetListingsDOM: function () {
        let petListings;
        let i = st.totalPets - st.petsVisible - 1;
        let threshold;

        if (st.filter === null) {
          threshold = i - st.numToLoad < 0 ? -1 : i - st.numToLoad;
        } else {
          threshold = -1;
        }

        petListings = "<div class='row sqs-row'>";

        for (i; i > threshold; i--) {
          // Check if the index is out of bounds or pet has no photo
          if (!st.pets[i] || st.pets.length === 0 || !st.pets[i].photoUrl) {
            console.log(`No pets found or no photo available for the given index ${i}.`);
            continue;
          }
          if (st.filter === null || st.filter === st.pets[i].type) {
            petListings += st.getPetDOM(i);
          } else if (st.filter === "Other") {
            if (st.pets[i].type !== "Dog" && st.pets[i].type !== "Cat") {
              petListings += st.getPetDOM(i);
            }
          }

          st.petsVisible += 1;
        }

        petListings += "</div>";

        return petListings;
      },

      getPetDOM: function (i) {
        const pet = st.pets[i];
        let petDOM;

        petDOM = "<div class='col sqs-col-4 span-4' style='opacity: 0; transform: translateY(10px); transition: opacity 0.3s ease, transform 0.3s ease;'>";
        petDOM += "<div class='sqs-block image-block html-block'>";
        petDOM += "<div class='petfinder__img-wrapper' style='aspect-ratio: 4/3; overflow: hidden;'>";
        petDOM += "<img alt='" + pet.name + "' src='" + pet.photoUrl + "' style='width: 100%; height: 100%; object-fit: cover; transition: transform 0.2s ease;' onmouseover=\"this.style.transform='scale(1.03)'\" onmouseout=\"this.style.transform='scale(1)'\" />";
        petDOM += "</div>";
        petDOM += "<h3>" + pet.name + "</h3>";

        // Show breed, age, sex info
        const details = [pet.breed, pet.age, pet.sex].filter(Boolean).join(" · ");
        if (details) {
          petDOM += "<p><strong>" + details + "</strong></p>";
        }

        // Show short description with link to full bio
        if (pet.short_description) {
          petDOM += "<p>" + pet.short_description + " <a href='" + pet.url + "' target='_blank'>Full bio on Adopt-a-Pet</a></p>";
        } else {
          petDOM += "<p><a href='" + pet.url + "' target='_blank'>Full bio on Adopt-a-Pet</a></p>";
        }

        // Apply to adopt link
        petDOM += "<p style='text-align: center;'>";
        if (pet.type === "Dog") {
          petDOM += "<a href='/application' title='apply to adopt this pet'>Apply to adopt " + pet.name + "!</a>";
        } else if (pet.type === "Cat") {
          petDOM += "<a href='/application-2/' title='apply to adopt this pet'>Apply to adopt " + pet.name + "!</a>";
        } else {
          petDOM += "<a href='/faqs/' title='apply to adopt this pet'>Apply to adopt " + pet.name + "!</a>";
        }
        petDOM += "</p>";
        petDOM += "</div>";
        petDOM += "</div>";

        return petDOM;
      },

      setupDOM: function () {
        const filter =
            "<div id='js-petfinder__filters' class='sqs-block button-block sqs-block-button'></div>";
        const petCount = "<p id='js-petfinder__count' style='margin: 1em 0;'></p>";
        const content = "<div id='js-petfinder__content'></div>";

        st.$petsWrapper.append(filter);
        st.$petsWrapper.append(petCount);
        st.$petsWrapper.append(content);
      },

      updatePetCount: function () {
        if (!st.$petCount) {
          st.$petCount = $("#js-petfinder__count");
        }
        const filterLabel = st.filter === null ? "pets" :
          st.filter === "Dog" ? "dogs" :
          st.filter === "Cat" ? "cats" : "pets";
        st.$petCount.text("Showing " + st.petsToShow + " " + filterLabel);
      },

      addFilters: function () {
        const buttonStyle = "margin: 0 0.5em 0.5em 0; transition: background-color 0.2s ease, border-color 0.2s ease;";
        let filters = "<h3>Filter by:</h3>";
        filters +=
          "<button class='sqs-block-button-element--small' style='" + buttonStyle + "' data-animal='null'>All</button>";
        filters +=
          "<button class='sqs-block-button-element--small' style='" + buttonStyle + "' data-animal='Dog'>dogs</button>";
        filters +=
          "<button class='sqs-block-button-element--small' style='" + buttonStyle + "' data-animal='Cat'>cats</button>";

        st.$petsFilters.append(filters);

        st.bindFilterClicks();
      },

      bindFilterClicks: function () {
        st.filterBtns = st.$petsFilters.children("button");

        // Set "All" as initially active
        st.filterBtns.first().addClass("active");

        st.filterBtns.on("click", function (e) {
          const $target = $(e.target);
          st.filterBtns.removeClass("active");

          st.filter = $target.data("animal");
          $target.addClass("active");

          st.startFilters();
        });
      },

      startFilters: function () {
        st.$petsContent.find(".col").addClass("loaded");
        st.$petsContent.html(st.$loader);

        st.petsVisible = 0;

        if (st.filter === "Dog") {
          st.petsToShow = st.dogs.length;
        } else if (st.filter === "Cat") {
          st.petsToShow = st.cats.length;
        } else if (st.filter === "Other") {
          st.petsToShow = st.other.length;
        } else {
          st.petsToShow = st.pets.length;
        }

        st.addPets();
      },

      getErrorDOM: function () {
        let msg = "<h2>Unable to load pets right now</h2>";
        msg += "<p>Please try again later.</p>";
        msg += "<p><a href='https://www.adoptapet.com/adoption_rescue/83349' title='Social Tees on Adopt-a-Pet'>View our pets on Adopt-a-Pet &rarr;</a></p>";
        return msg;
      },

      getEmptyDOM: function () {
        const filterLabel = st.filter === "Dog" ? "dogs" :
          st.filter === "Cat" ? "cats" : "pets";
        let msg = "<p style='text-align: center; padding: 2em 0;'>";
        msg += "No " + filterLabel + " available right now. Check back soon!";
        msg += "</p>";
        return msg;
      },

      getLoaderDOM: function () {
        let loader = "<div class='loader__wrapper'>";
        loader +=
          "<img src='" + st.loaderURL + "' alt='loader' class='loader' />";
        loader += "</div>";

        return loader;
      },
    };

    $(function () {
      st.init();
    });
  })();
