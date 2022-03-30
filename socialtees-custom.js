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
      $filterBtns: null,
      $loader: null,
  
      isLoading: true,
      // PetFinder only shows 20 per page and we don't currently support pagination
      numToLoad: 20,
      totalPets: 0,
      petsVisible: 0,
      petsToShow: 0,
      filter: null,
  
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
        var footerHeight = st.$preFooter.outerHeight() + st.$footer.outerHeight();
  
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
        const pf = new petfinder.Client({
          apiKey: "eWWxBarOWYLSKqUEjBKb6B8k04QSaEdJz6WHhSeSQqNj35LC23",
          secret: "dQp8i1PIMXv49VABfqV8iNc9bwWtq9AEPa8nQZeN",
        });
  
        pf.animal
          .search({ organization: "NY835" })
          .then((res) => {
            st.pets = res.data.animals;
            st.totalPets = st.petsToShow = st.pets.length;
            st.processPets();
            st.setupDOM();
            st.$petsContent = $("#js-petfinder__content");
            st.$petsFilters = $("#js-petfinder__filters");
            st.addFilters();
            st.addPets();
          })
          .catch((err) => {
            st.$loader.remove();
            st.$petsWrapper.append(st.getErrorDOM());
          });
      },
  
      processPets: function () {
        if (st.pets === null) return;
  
        var i = 0;
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
        st.$petsContent.append(st.getPetListingsDOM());
  
        window.setTimeout(function () {
          if (!st.$petsFilters.hasClass("loaded")) {
            st.$petsFilters.addClass("loaded");
          }
          st.$loader.remove();
          st.$petsContent.find(".col").addClass("loaded");
          st.isLoading = false;
        }, 250);
      },
  
      getPetListingsDOM: function () {
        var petListings;
        var i = st.totalPets - st.petsVisible - 1;
        var threshold;
  
        if (st.filter === null) {
          threshold = i - st.numToLoad < 0 ? -1 : i - st.numToLoad;
        } else {
          threshold = -1;
        }
  
        petListings = "<div class='row sqs-row'>";
  
        for (i; i > threshold; i--) {
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
        var petDOM;
        var imgURL;
        var size = st.pets[i].size;
        var gender = st.pets[i].gender;
  
        // NOTE: Normally we'd use https://photos.petfinder.com/ as the prefix, but their cert
        // is invalid and fails to load at https. So instead, we bypass it and load cloudfront
        // directly.
        // https://github.com/petfinder-com/petfinder-js-sdk/issues/22
        imgURL =
          "https://dl5zpyw5k3jeb.cloudfront.net/photos/pets/" + st.pets[i].id + "/1/";
        petDOM = "<div class='col sqs-col-4 span-4'>";
        petDOM += "<div class='sqs-block image-block html-block'>";
        petDOM += "<div class='petfinder__img-wrapper'>";
        // TODO this weirdly linkifies all the text below it too wtf
        // petDOM +=
        //   "<a href='https://www.petfinder.com/petdetail/" + st.pets[i].id + "'>\n" +
        //   "<img alt='" + st.pets[i].name + "' src='" + imgURL + "'>\n"
        //   "</a><br>";
        petDOM += "<img alt='" + st.pets[i].name + "' src='" + imgURL + "' />"
        petDOM += "</div>";
        petDOM += "<h3>" + st.pets[i].name + "</h3>";
  
        petDOM += "<ul>";
        if (st.pets[i].breeds.primary) {
          petDOM += "<li>" + st.pets[i].breeds.primary;
          if (st.pets[i].breeds.secondary) {
            petDOM += " / " + st.pets[i].breeds.primary;
          }
          petDOM += "</li>";
        }
        petDOM += "<li>" + st.pets[i].age + "</li>";
        if (size !== undefined) {
          petDOM += "<li>" + size + "</li>";
        }
        if (gender !== undefined) {
          petDOM += "<li>" + gender + "</li>";
        }
        petDOM +=
        "<li><a href='https://www.petfinder.com/petdetail/" +
        st.pets[i].id +
        "' target='_blank'>Full bio on petfinder</a></li>";
        petDOM += "</ul>";
  
        if (st.pets[i].type === "Dog") {
          petDOM +=
            "<a href='/application' title='apply to adopt this pet'>Apply to adopt " + st.pets[i].name + "!</a>";
        } else if (st.pets[i].type === "Cat") {
          petDOM +=
            "<a href='/application-2/' title='apply to adopt this pet'>Apply to adopt " + st.pets[i].name + "!</a>";
        } else {
          petDOM +=
            "<a href='/faqs/' title='apply to adopt this pet'>Apply to adopt " + st.pets[i].name + "!</a>";
        }
        petDOM += "</div>";
        petDOM += "</div>";
  
        return petDOM;
      },
  
      setupDOM: function () {
        var filter =
          "<div id='js-petfinder__filters' class='sqs-block button-block sqs-block-button'></div>";
        var content = "<div id='js-petfinder__content'></div>";
  
        st.$petsWrapper.append(filter);
        st.$petsWrapper.append(content);
      },
  
      addFilters: function () {
        var filters = "<h3>Filter by:</h3>";
        filters +=
          "<button class='sqs-block-button-element--small' data-animal='null'>All</button>";
        filters +=
          "<button class='sqs-block-button-element--small' data-animal='Dog'>dogs</button>";
        filters +=
          "<button class='sqs-block-button-element--small' data-animal='Cat'>cats</button>";
        // "Other" isn't supported right now
        // filters +=
        //  "<button class='sqs-block-button-element--small' data-animal='Other'>other</button>";
  
        st.$petsFilters.append(filters);
  
        st.bindFilterClicks();
      },
  
      bindFilterClicks: function () {
        st.filterBtns = st.$petsFilters.children("button");
  
        st.filterBtns.on("click", function (e) {
          var $target = $(e.target);
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
          st.petsToShow === st.cats.length;
        } else if (st.filter === "Other") {
          st.petsToShow === st.other.length;
        } else {
          st.petsToShow = st.pets.length;
        }
  
        st.addPets();
      },
  
      getErrorDOM: function () {
        var msg = "<h1>Looks like there was an error</h1>";
        msg +=
          "<p>See our adoptable pets <a href='https://www.petfinder.com/pet-search?shelterid=NY835' title='Social tees on Petfinder'>here.</a></p>";
  
        return msg;
      },
  
      getLoaderDOM: function () {
        var loader = "<div class='loader__wrapper'>";
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
  
