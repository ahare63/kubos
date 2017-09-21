/*
 * Copyright (C) 2017 Kubos Corporation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
/**
 * Functions for calling/handling/parsing the EnableLine method exposed
 * by the Power Manager.
 *
 * org.KubOS.PowerManager.EnableLine
 */

#include <dbus/dbus.h>
#include <stdlib.h>
#include "evented-control/ecp.h"
#include "evented-control/messages.h"

tECP_Error on_enable_line_parser(tECP_Context * context, DBusMessage * message,
                                 struct _tECP_MessageHandler * handler)
{
    DBusMessage *                    reply = NULL;
    uint8_t                          line  = -1;
    tECP_EnableLine_MessageHandler * line_handler
        = (tECP_EnableLine_MessageHandler *) handler;

    dbus_message_get_args(message, NULL, DBUS_TYPE_INT16, &line);
    printf("on_enable_line_parser line %d\n", line);

    line_handler->cb(line);

    reply = dbus_message_new_method_return(message);
    dbus_connection_send(context->connection, reply, NULL);
    dbus_message_unref(reply);
}

tECP_Error on_enable_line(tECP_Context * context, enable_line_cb cb)
{
    tECP_EnableLine_MessageHandler * enable_line_handler
        = malloc(sizeof(*enable_line_handler));
    enable_line_handler->super.interface = POWER_MANAGER_INTERFACE;
    enable_line_handler->super.member    = POWER_MANAGER_ENABLE_LINE;
    enable_line_handler->super.parser    = &on_enable_line_parser;
    enable_line_handler->super.next      = NULL;
    enable_line_handler->cb              = cb;

    return ECP_Add_Message_Handler(context, &enable_line_handler->super);
}

tECP_Error enable_line(tECP_Context * context, uint8_t line)
{
    DBusMessage * message = NULL;

    message = dbus_message_new_method_call(
        POWER_MANAGER_INTERFACE, POWER_MANAGER_PATH, POWER_MANAGER_INTERFACE,
        POWER_MANAGER_ENABLE_LINE);

    dbus_message_append_args(message, DBUS_TYPE_INT16, &line,
                             DBUS_TYPE_INVALID);

    return ECP_Call(context, message);
}