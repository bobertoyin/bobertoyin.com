function getRandomInt(min, max) {
    min = Math.ceil(min);
    max = Math.floor(max);
    return Math.floor(Math.random() * (max - min) + min);
}

document.addEventListener('DOMContentLoaded', () => {
    fetch('quotes.json')
        .then(response => response.json())
        .then(quotes => {
            let random_quote = quotes[getRandomInt(0, quotes.length)];
            document.getElementById("random-quote").innerHTML = `
                <p>
                    <a href="${random_quote.source}" target="_blank">
                        <strong>${random_quote.text}</strong>
                    </a>
                        —<em>${random_quote.speaker}</em>
                </p>
            `;
        });
});
