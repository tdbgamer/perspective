# Schema and column types

The mapping of a `Table`'s column names to data types is referred to as a
`schema`. Each column has a unique name and a single data type, one of

-   `float`
-   `integer`
-   `boolean`
-   `date`
-   `datetime`
-   `string`

A `Table` schema is fixed at construction, either by explicitly passing a schema
dictionary to the `Client::table` method, or by passing _data_ to this method
from which the schema is _inferred_ (if CSV or JSON format) or inherited (if
Arrow).

## Type inference

When passing data directly to the `crate::Client::table` constructor, the type
of each column is inferred automatically. In some cases, the inference algorithm
may not return exactly what you'd like. For example, a column may be interpreted
as a `datetime` when you intended it to be a `string`, or a column may have no
values at all (yet), as it will be updated with values from a real-time data
source later on. In these cases, create a `table()` with a _schema_.

Once the `Table` has been created with a schema, further `update()` calls will
no longer perform type inference, so columns must only include values supported
by the column's `ColumnType`.
