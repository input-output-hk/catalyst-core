# Example Enhanced Markup

This markup is available to enhance the presentation of the book.

```admonish info
A beautifully styled message.
```

```admonish example
My example is the best!
```

```admonish
A plain note.
```

```admonish warning title="Data loss"
The following steps can lead to irrecoverable data corruption.
```

```admonish success title=""
This will take a while, go and grab a drink of water.
```

```admonish tip title="_Referencing_ and <i>dereferencing</i>"
The opposite of *referencing* by using `&` is *dereferencing*, which is
accomplished with the <span style="color: hotpink">dereference operator</span>, `*`.
```

~~~admonish bug
This syntax won't work in Python 3:
```python
print "Hello, world!"
```
~~~

### Example Rendered Diagrams

```kroki-plantuml
@startuml
A --|> B
@enduml
```

~~~admonish title="Don't Click Me" collapsible=true

```kroki-mermaid
%%{init: {'theme':'forest'}}%%
graph TD
  A[ Anyone ] -->|Can help | B( Go to https://github.com/input-output-hk/catalyst-core )
  click B "https://github.com/input-output-hk/catalyst-core"

  B --> C{ How to contribute? }
  C --> D[ Reporting bugs ]
  C --> E[ Sharing ideas ]
  C --> F[ Advocating ]
```

~~~
