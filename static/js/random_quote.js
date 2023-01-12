function getRandomInt(min, max) {
    min = Math.ceil(min);
    max = Math.floor(max);
    return Math.floor(Math.random() * (max - min) + min);
}

function randomNoRepeats(array) {
    var copy = array.slice(0);
    return function() {
      if (copy.length < 1) { copy = array.slice(0); }
      var index = Math.floor(Math.random() * copy.length);
      var item = copy[index];
      copy.splice(index, 1);
      return item;
    };
}

function generateRandomQuote() {
    let randomQuote = quoteGenerator();
    document.getElementById("random-quote").innerHTML = `
        <p>
            <strong>${randomQuote.text}</strong>
            <br>
            <br>
            <a href="${randomQuote.source}" target="_blank">
                ${randomQuote.speaker}
            </a>
            
        </p>
    `;
}

var quoteGenerator;

fetch('quotes.json')
    .then(response => response.json())
    .then(quotes => {
        quoteGenerator = randomNoRepeats(quotes);
        generateRandomQuote();
});
