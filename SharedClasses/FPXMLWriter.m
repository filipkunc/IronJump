//
//  FPXMLWriter.m
//  IronJumpLevelEditor
//
//  Created by Filip Kunc on 9/14/10.
//  For license see LICENSE.TXT
//

#import "FPXMLWriter.h"

@implementation FPXMLWriter

- (id)init
{
    self = [super init];
    if (self) 
    {
        string = [[NSMutableString alloc] init];
        [string appendFormat:@"\357\273\277%@", @"<?xml version=\"1.0\" encoding=\"utf-8\"?>"];
        depth = 0;
        closedTwiceOrMore = NO;
    }
    return self;
}

- (void)dealloc 
{
    [string release];
    [super dealloc];
}

- (NSData *)data
{
    return [string dataUsingEncoding:NSUTF8StringEncoding];
}

- (void)openElement:(NSString *)elementName
{
    [string appendString:@"\n"];
    for (int i = 0; i < depth; i++)
        [string appendString:@"  "]; // 2 spaces per depth

    [string appendFormat:@"<%@>", elementName];
    depth++;
    
    closedTwiceOrMore = NO;
}

- (void)closeElement:(NSString *)elementName
{
    depth--;
    if (closedTwiceOrMore)
    {
        [string appendString:@"\n"];
        for (int i = 0; i < depth; i++)
            [string appendString:@"  "]; // 2 spaces per depth
    }    
    [string appendFormat:@"</%@>", elementName];
    closedTwiceOrMore = YES;
}

- (void)writeFloatValue:(float)value
{
    [string appendFormat:@"%f", value];
}

- (void)writeIntValue:(int)value
{
    [string appendFormat:@"%i", value];
}

- (void)writeElementWithName:(NSString *)elementName floatValue:(float)value
{
    [self openElement:elementName];
    [self writeFloatValue:value];
    [self closeElement:elementName];
}

- (void)writeElementWithName:(NSString *)elementName intValue:(int)value
{
    [self openElement:elementName];
    [self writeIntValue:value];
    [self closeElement:elementName];
}

@end
