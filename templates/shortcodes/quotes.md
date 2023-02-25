{% set quotes = load_data(path="static/quotes.json") %}
{% for quote in quotes %}
> {{ quote.text }}
>
>
> [{{ quote.speaker }}]({{ quote.source }})
{% endfor %}
