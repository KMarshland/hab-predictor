/*
 * Copyright 2005-2017 ECMWF.
 *
 * This software is licensed under the terms of the Apache Licence Version 2.0
 * which can be obtained at http://www.apache.org/licenses/LICENSE-2.0.
 *
 * In applying this licence, ECMWF does not waive the privileges and immunities granted to it by
 * virtue of its status as an intergovernmental organisation nor does it submit to any jurisdiction.
 */

/*
 *
 */


#include "grib_api_internal.h"

#define NUMBER(x) (sizeof(x)/sizeof(x[0]))

/* This file is generated my ./make_class.pl */
#include "grib_nearest_class.h"

struct table_entry
{
  char                  *type;
  grib_nearest_class   **cclass;
};

static struct table_entry table[] =
{
  /* This file is generated my ./make_class.pl */
#include "grib_nearest_factory.h"
};

grib_nearest* grib_nearest_factory( grib_handle* h, grib_arguments* args)
{
  int i;
  int ret = GRIB_SUCCESS;
  char* type = (char*)grib_arguments_get_name(h,args,0);

  for(i = 0; i < NUMBER(table) ; i++)
    if(strcmp(type,table[i].type) == 0)
    {
      grib_nearest_class* c = *(table[i].cclass);
      grib_nearest*       it = (grib_nearest*) grib_context_malloc_clear(h->context,c->size);
      it->cclass              = c;
      ret = GRIB_SUCCESS;
      ret = grib_nearest_init(it,h,args);
      if(ret ==  GRIB_SUCCESS) return it;
      grib_context_log(h->context,GRIB_LOG_ERROR,"grib_nearest_factory: error %d instantiating nearest %s", ret, table[i].type);
      grib_nearest_delete(it);
      return NULL;
    }

  grib_context_log(h->context,GRIB_LOG_ERROR,"grib_nearest_factory : Unknown type : %s for nearest", type);

  return NULL;
}
