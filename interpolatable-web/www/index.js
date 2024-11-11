const interpolatableWorker = new Worker(
  new URL("./webworker.js", import.meta.url)
);

const COLORS = [
  "#4269d0", // blue
  "#efb118", // orange
  "#ff725c", // red
  "#6cc5b0", // cyan
  "#3ca951", // green
  "#ff8ab7", // pink
  "#a463f2", // purple
  "#97bbf5", // light blue
  "#9c6b4e", // brown
  "#9498a0"  // gray
]
const SVG = require("@svgdotjs/svg.js");

function problem2string(problem) {
  if (problem.type == "PathCount") {
    return `Path count mismatch: ${problem.count_1} in ${problem.master_1_name} vs ${problem.count_2} in ${problem.master_2_name}`;
  }
  if (problem.type == "NodeCount") {
    return `Node count mismatch: ${problem.count_1} in ${problem.master_1_name} vs ${problem.count_2} in ${problem.master_2_name}`;
  }
  if (problem.type == "NodeIncompatibility") {
    return `Incompatible nodes: Node ${problem.node} <span class="contour-${problem.contour}">contour ${problem.contour}</span> is ${p.is_control_1 ? 'off-curve' : 'on-curve'} in ${problem.master_1_name} vs ${p.is_control_2 ? 'off-curve' : 'on-curve'} in ${problem.master_2_name}`;
  }
  if (problem.type == "ContourOrder") {
    return `Contour order mismatch: <span class="contour-${problem.order_1}">${problem.order_1}</span> in ${problem.master_1_name} matches with <span class="contour-${problem.order_2}">${problem.order_2}</span> in ${problem.master_2_name}`;
  }
  if (problem.type == "WrongStartPoint") {
    let reverse = "";
    if (problem.reverse) {
      reverse = " (and the contour should be reversed)";
    }
    return `Wrong start point: <span class="contour-${problem.contour}">contour ${problem.contour}</span> in ${problem.master_2_name} should start at node ${problem.proposed_point} ${reverse}`;
  }
  if (problem.type == "Overweight") {
    return `Overweight: <span class="contour-${problem.contour}">contour ${problem.contour}</span> becomes overweight`;
  }
  if (problem.type == "Underweight") {
    return `Underweight: <span class="contour-${problem.contour}">contour ${problem.contour}</span> becomes underweight`;
  }
  if (problem.type == "Kink") {
    return `Kink: <span class="contour-${problem.contour}">contour ${problem.contour}</span> has a kink`;
  }
}

jQuery.fn.shake = function (interval, distance, times) {
  interval = typeof interval == "undefined" ? 100 : interval;
  distance = typeof distance == "undefined" ? 10 : distance;
  times = typeof times == "undefined" ? 3 : times;
  var jTarget = $(this);
  jTarget.css("position", "relative");
  for (var iter = 0; iter < times + 1; iter++) {
    jTarget.animate(
      {
        left: iter % 2 == 0 ? distance : distance * -1,
      },
      interval
    );
  }
  return jTarget.animate(
    {
      left: 0,
    },
    interval
  );
};

class Interpolatable {
  constructor() {
    this.font = null;
  }

  get cssStyle() {
    return document.styleSheets[0].cssRules[0].style;
  }

  setVariationStyle(variations, element) {
    element.css("font-variation-settings", variations);
  }

  dropFile(files, element) {
    if (!files[0].name.match(/\.[ot]tf$/i)) {
      $(element).shake();
      return;
    }
    var style = this.cssStyle;
    window.thing = files[0];
    $("#filename").text(files[0].name);
    style.setProperty("src", "url(" + URL.createObjectURL(files[0]) + ")");
    var reader = new FileReader();
    let that = this;
    reader.onload = function (e) {
      let u8 = new Uint8Array(this.result);
      that.font = u8;
      that.letsDoThis();
    };
    reader.readAsArrayBuffer(files[0]);
  }

  progress_callback(message) {
    console.log("Got message", message);
    if ("ready" in message) {
      $("#bigLoadingModal").hide();
      $("#startModal").show();
    } else if ("results" in message) {
      $("#spinnerModal").hide();
      this.renderResults(message.results);
    }
  }

  renderResults(results) {
    if (!Object.keys(results).length) {
      $("#cupcake").removeClass("d-none");
      console.log("cupcake")
      return;
    }
    let ix = 0;
    for (let [glyph, problems] of Object.entries(results)) {
      ix += 1;
      var thispill = $(`
        <button
          class="nav-link"
          id="v-pills-${ix}-tab"
          type="button"
          role="tab"
          aria-controls="v-pills-${ix}">${glyph}</button>
      `);
      thispill.data("problemset", problems);
      $("#glyphlist").append(thispill);
      thispill.on("click", (el) => {
         this.renderProblemSet($(el.target))
         $(el.target).siblings().removeClass("active");
          $(el.target).addClass("active");
    });
    }
    $("#glyphlist button:first-child").tab("show").addClass("active").trigger("click");
  }

  renderSvg(container, outlines) {
    let svg = SVG.adopt(container[0]);
    svg.clear();
    let group = svg.group();
    let shadowgroup = group.group();
    let outlinegroup = group.group();
    for (var [index, contour] of outlines.entries()) {
      outlinegroup.path(contour).fill("none").stroke({ width: 5, color: COLORS[index % COLORS.length] });
    }
    shadowgroup.path(outlines.join(" ")).fill("#000").opacity(0.25).attr({'fill-rule': 'even-odd'});
    let bbox = group.bbox();
    svg.viewbox(bbox);
    group.transform({ flip: "y" });
}

  renderProblemSet(el) {
    let result = $("#v-pills-tabContent div");
    result.empty();
    var problemSet = el.data("problemset");
    console.log(problemSet);
    for (var problems of problemSet) {
      // A set of problems between two masters
      let problem_html = $(
        `<div class="row">
          <div class="col-4">
            <h3>${problems["default_name"]}</h3>
            <svg class='beforesvg'></svg>
          </div>
          <div class="col-4">
            <h3>${problems["midway_location"]}</h3>
            <svg class='midwaysvg'></svg>
          </div>
          <div class="col-4">
            <h3>${problems["master_name"]}</h3>
            <svg class='aftersvg'></svg>
          </div>
          </div>
        `);
      this.renderSvg(problem_html.find(".beforesvg"), problems["default_outline"]);
      this.renderSvg(problem_html.find(".midwaysvg"), problems["midway_outline"]);
      this.renderSvg(problem_html.find(".aftersvg"), problems["outline"]);
      result.append(problem_html);
      let list = $("<ul></ul>");
      for (var problem of problems["problems"]) {
        list.append(`<li>${problem2string(problem)}</li>`);
      }
      result.append(list);
    }
  }

  letsDoThis() {
    $("#startModal").hide();
    $("#spinnerModal").show();
    interpolatableWorker.postMessage({
      font: this.font,
    });
  }
}

$(function () {
  window.interpolatable = new Interpolatable();
  interpolatableWorker.onmessage = (e) =>
    window.interpolatable.progress_callback(e.data);
  $("#bigLoadingModal").show();

  $(".fontdrop").on("dragover dragenter", function (e) {
    e.preventDefault();
    e.stopPropagation();
    $(this).addClass("dragging");
  });
  $(".fontdrop").on("dragleave dragend", function (e) {
    $(this).removeClass("dragging");
  });

  $(".fontdrop").on("drop", function (e) {
    console.log("Drop!");
    $(this).removeClass("dragging");
    if (
      e.originalEvent.dataTransfer &&
      e.originalEvent.dataTransfer.files.length
    ) {
      e.preventDefault();
      e.stopPropagation();
      interpolatable.dropFile(e.originalEvent.dataTransfer.files, this);
    }
  });

  // Add colors to CSS
  for (let [ix, color] of COLORS.entries()) {
    document.styleSheets[0].insertRule(`.contour-${ix} { color: ${color}; }`, 0);
  }
});
