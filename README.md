# EZProxy: Keyboard Shortcuts for Your Address Bar

ezproxy replaces your browser's default search engine in your address bar and lets you specify
keyboard shortcuts that you can use to quickly navigate to frequently-used sites. For example,
I can type `m` to go to GMail, `c` to go to calendar, and `npm $package_name` to search for packages.

ezproxy also supports a fallback argument so that you can fall back to your search engine of choice in lieu
of a keyboard shortcut.

Think of ezproxy like a URL shortener, but without having to type the URL. And it supports arguments. Companies
like Google and Meta have internal tools with similar functionality, and I wanted something like this for personal
use because I work from the keyboard and find it really productive to be able to get to where I need to go quickly,
and without leaving my keyboard. Chrome's custom search engine ability is helpful, but suffers from the fact that you have to tab-complete in order to use it, and it always expects arguments vs. simply issuing redirects.

# Setup

## Install ezproxy

Download the proper binary from https://github.com/traviskaufman/ezproxy/releases

## Create a config file

Copy the following into a file `ezproxy.txt`

```
m = https://gmail.com/
c = https://calendar.google.com/
yt = https://youtube.com/results?search_query={ARGS}
npm = https://npmjs.com/search?q={ARGS}
_ = https://www.google.com/search?q={ALL}
```

## Run ezproxy

Run

```sh
/path/to/ezproxy /path/to/ezproxy.txt
```

This will start a server on port `5050`. If you need to change the port, you can use the `--port` flag.

## Change your browser's default search engine to ezproxy

### In Google Chrome

Navigate to chrome://settings/searchEngines, scroll to where it says "Site search", and click the "Add" button.

Type the following into the dialog

- Search Engine: EZProxy
- Shortcut: ez
- URL: http://localhost:5050?q=%s

> NOTE: If you changed the port above from 5050, be sure to edit that in the URL above.

Then, hit save. Locate the search engine record, click the 3-dot menu on the right hand side of the record,
and select "Make Default".

You should now be able to use EZProxy. For more info, see https://zapier.com/blog/add-search-engine-to-chrome/

### In Firefox

See https://superuser.com/a/7336

# Adding Shortcuts

You add shortcuts through a **config**. The config file is a simple text format that looks like this:

```
<shortcut> = <url>
```

The shortcut can be any text except for whitespace. Note that (because this product is early) _there should only be one space between the `=` sign on either side_.

So for example, if you have:

```
m = https://gmail.com/
```

It means that when you type `m` into the address bar, you'll go to GMail.

It's recommended to copy over the `example-configs/simple.txt` to get started, and modify from there.

## Shortcut Arguments

EZProxy understands two different special tokens in its config:

## {ARGS}

`{ARGS}` will substitute everything after the **command**, e.g. the first phrase you type into the address bar.

If you have

```
npm = https://npmjs.com/search?q={ARGS}
```

And you type

```
npm file finder
```

You'll navigate to https://npmjs.com/search?q=file%20finder

## {ALL}

Sometimes it can be useful to have the entire string and arguments all together. You can use `{ALL}` for this.

If you have

```
tf = https://www.tensorflow.org/s/results/?q={ALL}
```

And you type

```
tf keras.layers.GRU
```

You'll navigate to https://www.tensorflow.org/s/results/?q=tf%20keras.layers.GRU

## Fallback shortcut

Adding a `_` fallback shortcut to your config is highly recommended, so that you can still do basic searching. For example:

```
_ = https://www.google.com/search?q={ALL}
```

## (Advanced) Adding Shortcuts in code

If you're feeling ambitious or want some extra functionality, you can add shortcuts in code by cloning this repo and
extending `src/rules.rs`. For example, here's how you could write a `yt` shortcut that takes you to the youtube home
page without any arguments, but takes you to the search page if arguments are provided.

```rs
#[derive(Default)]
pub struct YouTubeRule;
impl Rule for YouTubeRule {
  fn produce_uri(&self, _cmd: &str, args: &Vec<String>) -> Result<Uri, String> {
    let builder = Uri::builder().scheme("https").authority("youtube.com");

    let res = match args[..] {
      [] => builder.path_and_query("/").build(),
      _ => {
        let encoded = urlencoding::encode(&args.join(" ")).into_owned();
        builder
          .path_and_query(format!("/results?search_query={}", encoded))
          .build()
      }
    };

    res.map_err(|e| format!("Error producing URI: {}", e))
  }
}
```

then edit `src/main.rs` to add the rule
