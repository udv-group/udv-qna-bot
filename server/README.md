## API Design "gotchas"
- Since frontend is using HTMX, a lot of endpoints just render specific parts of html. For example, every row in a table is its own template. This allows to append rows to a table without the need to rerender the whole page.
- In order to get the file server working, kinda weird strategy is employed. First, `ServeDir` service is nested to `/static` path. The endpoint, that is getting called from the client, sends back a redirect in order to trigger browser native file download interaction. 
- Reordering is not interactive, i.e. user sorts the table in the desired way, then saves the changes. Saves a lot of headache and redrawing. 